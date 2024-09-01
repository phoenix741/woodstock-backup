import { Injectable, Logger } from '@nestjs/common';
import { Job } from 'bullmq';
import { instanceToPlain, plainToInstance } from 'class-transformer';
import {
  catchError,
  concatMap,
  defer,
  from,
  lastValueFrom,
  map,
  Observable,
  of,
  startWith,
  tap,
  throttleTime,
} from 'rxjs';
import {
  ABORTABLE_QUEUE_TASK_PRIORITY,
  QueueGroupTasks,
  QueueSubTask,
  QueueTaskContext,
  QueueTaskPriority,
  QueueTaskProgression,
  QueueTasks,
  QueueTasksInformations,
  QueueTaskState,
  QUEUE_TASK_FAILED_STATE,
  TASK_PRIORITY_ORDER,
} from './queue-tasks.model';

@Injectable()
export class QueueTasksService {
  #logger = new Logger(QueueTasksService.name);

  async executeTasksFromJob<JobData, Context>(
    job: Job<JobData>,
    informations: QueueTasksInformations<Context>,
    callback?: (informations: QueueTasksInformations<Context>) => Promise<void>,
  ): Promise<QueueTasksInformations<Context>> {
    if (typeof job.progress === 'object') {
      informations.tasks = this.deserializeBackupTask(job.progress);
    }
    job.updateProgress(this.serializeBackupTask(informations.tasks));

    const { context, tasks } = informations;

    await lastValueFrom(
      this.executeTasks(tasks, context).pipe(
        throttleTime(1000, undefined, { leading: true, trailing: true }), // TODO: Conf // When trailing is true, the throttle end arrive before it complete
        concatMap(async (task) => {
          job.updateProgress(this.serializeBackupTask(task));

          if (callback) {
            await callback(informations);
          }

          return task;
        }),
      ),
    );

    job.updateProgress(this.serializeBackupTask(informations.tasks));
    return informations;
  }

  serializeBackupTask(tasks: QueueTasks): object {
    return instanceToPlain(tasks);
  }

  deserializeBackupTask(data: object): QueueTasks {
    return plainToInstance(QueueTasks, data);
  }

  executeTasks<Context>(task: QueueTasks, context: QueueTaskContext<Context>): Observable<QueueTasks> {
    const subtasks = task.subtasks;
    return from(TASK_PRIORITY_ORDER).pipe(
      concatMap((priority) => this.#executeSubtasks(task, subtasks, context, priority)),
    );
  }

  #executeSubtasks<Context>(
    primaryTask: QueueTasks,
    subtasks: (QueueSubTask | QueueGroupTasks)[],
    context: QueueTaskContext<Context>,
    priority: QueueTaskPriority,
  ): Observable<QueueTasks> {
    return from(subtasks).pipe(
      concatMap((subtask) => {
        if (subtask instanceof QueueGroupTasks) {
          return this.#executeSubtasks(primaryTask, subtask.subtasks, context, priority);
        }

        if (subtask.priority === priority) {
          if (
            ABORTABLE_QUEUE_TASK_PRIORITY.includes(subtask.priority) &&
            QUEUE_TASK_FAILED_STATE.includes(primaryTask.state)
          ) {
            // If the primary task is aborted or failed, we stop the execution
            subtask.state = QueueTaskState.ABORTED;
            return of(primaryTask);
          }
          return this.#executeSubtask(
            primaryTask,
            subtask,
            context,
            QUEUE_TASK_FAILED_STATE.includes(primaryTask.state),
          );
        }

        return of(primaryTask);
      }),
    );
  }

  #executeSubtask<Context>(
    primaryTask: QueueTasks,
    subtask: QueueSubTask,
    context: QueueTaskContext<Context>,
    isFailing: boolean,
  ): Observable<QueueTasks> {
    if (subtask.state !== QueueTaskState.WAITING) {
      if (subtask.state === QueueTaskState.RUNNING) {
        this.#logger.error(`Task ${subtask.taskName} previously running: fail the job`, subtask.taskName);
        subtask.state = QueueTaskState.FAILED;
      }

      return of(primaryTask);
    }

    const command = context.commands.get(subtask.taskName);
    if (!command) {
      throw new Error(`No command found for task ${subtask.taskName}`);
    }
    const launchCommand = defer(() => command(context, subtask.localContext, isFailing));

    subtask.progression = new QueueTaskProgression();
    subtask.state = QueueTaskState.RUNNING;

    this.#logger.debug?.(`Start task ${subtask.taskName} `, subtask.taskName);
    return launchCommand.pipe(
      startWith(subtask.progression),
      map((progression) => {
        subtask.progression = progression ?? subtask.progression;
        return primaryTask;
      }),
      catchError((err) => {
        this.#logger.error(`Error while executing ${subtask.taskName} ${err.message}`, err.stack, subtask.taskName);
        subtask.state = QueueTaskState.FAILED;
        return of(primaryTask);
      }),
      tap({
        complete: () => {
          if (!QUEUE_TASK_FAILED_STATE.includes(subtask.state)) {
            subtask.progression = subtask.progression || new QueueTaskProgression();
            subtask.state = QueueTaskState.SUCCESS;
          }
          this.#logger.debug?.(`End task with state ${subtask.state}`, subtask.taskName);
          return primaryTask;
        },
      }),
    );
  }
}
