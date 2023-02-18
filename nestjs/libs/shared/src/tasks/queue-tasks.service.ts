import { Injectable } from '@nestjs/common';
import { instanceToPlain, plainToInstance } from 'class-transformer';
import { catchError, concatMap, defer, from, map, Observable, of, startWith, tap } from 'rxjs';
import {
  ABORTABLE_QUEUE_TASK_PRIORITY,
  QueueGroupTasks,
  QueueSubTask,
  QueueTaskContext,
  QueueTaskPriority,
  QueueTaskProgression,
  QueueTasks,
  QueueTaskState,
  QUEUE_TASK_FAILED_STATE,
  TASK_PRIORITY_ORDER,
} from './queue-tasks.model';

@Injectable()
export class QueueTasksService {
  serializeBackupTask(tasks: QueueTasks): object {
    return instanceToPlain(tasks);
  }

  deserializeBackupTask(data: object): QueueTasks {
    return plainToInstance(QueueTasks, data, { enableImplicitConversion: true });
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
          return this.#executeSubtask(primaryTask, subtask, context);
        }

        return of(primaryTask);
      }),
    );
  }

  #executeSubtask<Context>(
    primaryTask: QueueTasks,
    subtask: QueueSubTask,
    context: QueueTaskContext<Context>,
  ): Observable<QueueTasks> {
    if (subtask.state !== QueueTaskState.WAITING) {
      if (subtask.state === QueueTaskState.RUNNING) {
        context.logger.error(`Task ${subtask.taskName} previously running: fail the job`, subtask.taskName);
        subtask.state = QueueTaskState.FAILED;
      }

      return of(primaryTask);
    }

    const command = context.commands.get(subtask.taskName);
    if (!command) {
      throw new Error(`No command found for task ${subtask.taskName}`);
    }
    const launchCommand = defer(() => command(context, subtask.localContext));

    subtask.progression = new QueueTaskProgression();
    subtask.state = QueueTaskState.RUNNING;

    context.logger.log(`Start task ${subtask.taskName} `, subtask.taskName);
    return launchCommand.pipe(
      startWith(subtask.progression),
      map((progression) => {
        subtask.progression = progression || subtask.progression;
        return primaryTask;
      }),
      catchError((err) => {
        context.logger.error(`Error while executing ${subtask.taskName} ${err.message}`, err.stack, subtask.taskName);
        subtask.state = QueueTaskState.FAILED;
        return of(primaryTask);
      }),
      tap({
        complete: () => {
          if (!QUEUE_TASK_FAILED_STATE.includes(subtask.state)) {
            subtask.progression = subtask.progression || new QueueTaskProgression();
            subtask.state = QueueTaskState.SUCCESS;
          }
          context.logger.log(`End task with state ${subtask.state}`, subtask.taskName);
          return primaryTask;
        },
      }),
    );
  }
}
