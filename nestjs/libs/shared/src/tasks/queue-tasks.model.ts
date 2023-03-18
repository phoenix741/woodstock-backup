import { LoggerService } from '@nestjs/common';
import { Exclude, Transform, Type } from 'class-transformer';
import { Observable } from 'rxjs';
import { bigIntTransformation } from '../utils/transform.utils';

/**
 * Define the priority of a task
 */
export enum QueueTaskPriority {
  INITIALISATION = 'INITIALISATION', // Prepare backup directory
  PRE_PROCESSING = 'PRE_PROCESSING', // Connection to the host
  PROCESSING = 'PROCESSING', // Getting the file list, and chunks
  POST_PROCESSING = 'POST_PROCESSING', // Close the connection
  FINALISATION = 'FINALISATION', // Compact, Ref count
}

export const ABORTABLE_QUEUE_TASK_PRIORITY = [
  QueueTaskPriority.INITIALISATION,
  QueueTaskPriority.PRE_PROCESSING,
  QueueTaskPriority.PROCESSING,
];

export const TASK_PRIORITY_ORDER = [
  QueueTaskPriority.INITIALISATION,
  QueueTaskPriority.PRE_PROCESSING,
  QueueTaskPriority.PROCESSING,
  QueueTaskPriority.POST_PROCESSING,
  QueueTaskPriority.FINALISATION,
];

/**
 * Define the state of the task in the queue
 */
export enum QueueTaskState {
  WAITING = 'WAITING',
  RUNNING = 'RUNNING',
  SUCCESS = 'SUCCESS',
  ABORTED = 'ABORTED',
  FAILED = 'FAILED',
}

export const QUEUE_TASK_FAILED_STATE = [QueueTaskState.ABORTED, QueueTaskState.FAILED];

export const QUEUE_TASK_SUCCESS_STATE = [QueueTaskState.SUCCESS];

/**
 * Define the command to execute for a subtask
 */
export type QueueSubTaskCommand<GlobalContext, LocalContext> = (
  context: QueueTaskContext<GlobalContext>,
  localContext: LocalContext,
) => Observable<QueueTaskProgression> | Promise<QueueTaskProgression | void>;

/**
 * Define the context of tasks
 */
export class QueueTaskContext<GlobalContext> {
  commands = new Map<string, QueueSubTaskCommand<GlobalContext, TaskLocalContext>>();

  constructor(public readonly globalContext: GlobalContext, public readonly logger: LoggerService) {}
}

export class TaskLocalContext {
  host?: string;
  number?: number;

  command?: string;
  shares?: string[];
  @Type(() => Buffer)
  includes?: Buffer[];
  @Type(() => Buffer)
  excludes?: Buffer[];
  @Type(() => Buffer)
  sharePath?: Buffer;
}

/**
 * Define the progression of a task
 */
export class QueueTaskProgression {
  @Transform(bigIntTransformation)
  compressedFileSize = 0n;
  @Transform(bigIntTransformation)
  newCompressedFileSize = 0n;

  @Transform(bigIntTransformation)
  fileSize = 0n;
  @Transform(bigIntTransformation)
  newFileSize = 0n;

  newFileCount = 0;
  fileCount = 0;

  errorCount = 0;

  speed = 0;

  @Transform(bigIntTransformation)
  progressCurrent = 0n;

  @Transform(bigIntTransformation)
  progressMax = 0n;

  get percent(): number {
    if (this.progressMax) {
      return Number((this.progressCurrent * 100n) / this.progressMax);
    }
    return 0;
  }

  constructor(s: Partial<QueueTaskProgression> = {}) {
    Object.assign(this, s);
  }

  static merge(progressions: QueueTaskProgression[]): QueueTaskProgression {
    return progressions.reduce((acc, p) => acc.merge(p), new QueueTaskProgression());
  }

  merge(progression?: QueueTaskProgression): QueueTaskProgression {
    this.progressCurrent += progression?.progressCurrent ?? 0n;
    this.progressMax += progression?.progressMax ?? 0n;

    this.compressedFileSize += progression?.compressedFileSize ?? 0n;
    this.newCompressedFileSize += progression?.newCompressedFileSize ?? 0n;

    this.fileSize += progression?.fileSize ?? 0n;
    this.newFileSize += progression?.newFileSize ?? 0n;

    this.newFileCount += progression?.newFileCount ?? 0;
    this.fileCount += progression?.fileCount ?? 0;

    this.errorCount += progression?.errorCount ?? 0;

    this.speed += progression?.speed ?? 0;

    return this;
  }
}

abstract class AbstractQueueTask {
  protected __type: string;
  readonly localContext: TaskLocalContext;
}

/**
 * Define a subtask of a task
 */
export class QueueSubTask extends AbstractQueueTask {
  state = QueueTaskState.WAITING;

  @Type(() => QueueTaskProgression)
  progression = new QueueTaskProgression();

  readonly taskName: string;
  @Type(() => TaskLocalContext)
  readonly localContext: TaskLocalContext;
  readonly priority: QueueTaskPriority;

  constructor(taskName: string, localContext?: TaskLocalContext, priority = QueueTaskPriority.PROCESSING) {
    super();
    this.__type = 'subtask';
    this.taskName = taskName;
    this.localContext = localContext ?? {};
    this.priority = priority;
  }
}

/**
 * Define a group of subtasks
 */
export class QueueGroupTasks extends AbstractQueueTask {
  @Type(() => AbstractQueueTask, {
    discriminator: {
      property: '__type',
      subTypes: [
        { value: QueueSubTask, name: 'subtask' },
        { value: QueueGroupTasks, name: 'grouptask' },
      ],
    },
  })
  subtasks: (QueueSubTask | QueueGroupTasks)[] = [];
  readonly groupName: string;
  @Type(() => TaskLocalContext)
  readonly localContext: TaskLocalContext;

  constructor(groupName: string, localContext?: TaskLocalContext) {
    super();
    this.__type = 'grouptask';
    this.groupName = groupName;
    this.localContext = localContext ?? {};
  }

  add(...task: (QueueSubTask | QueueGroupTasks)[]) {
    this.subtasks.push(...task);
    return this;
  }

  @Exclude()
  get state(): QueueTaskState {
    const subtasks = this.subtasks;

    // Is Some tasks are running, the whole task is running
    const running = subtasks.some((subtask) =>
      [QueueTaskState.RUNNING, QueueTaskState.WAITING].includes(subtask.state),
    );

    // If some tasks are failed or aborted, the whole task is failed
    const failed = subtasks.some((subtask) => [QueueTaskState.FAILED, QueueTaskState.ABORTED].includes(subtask.state));

    if (running) {
      if (failed) {
        return QueueTaskState.ABORTED;
      }

      const started = subtasks.some((subtask) => [QueueTaskState.RUNNING].includes(subtask.state));

      const hasSuccess = subtasks.some((subtask) => [QueueTaskState.SUCCESS].includes(subtask.state));
      const hasWaiting = subtasks.some((subtask) => [QueueTaskState.WAITING].includes(subtask.state));
      if (started || (hasSuccess && hasWaiting)) {
        return QueueTaskState.RUNNING;
      }
      return QueueTaskState.WAITING;
    } else {
      if (failed) {
        return QueueTaskState.FAILED;
      }
      return QueueTaskState.SUCCESS;
    }
  }

  @Exclude()
  get progression(): QueueTaskProgression {
    return QueueTaskProgression.merge(this.subtasks.map((subtask) => subtask.progression));
  }
}

/**
 * Define a tasks with his subtasks
 */
export class QueueTasks extends QueueGroupTasks {}

export class QueueTasksInformations<GlobalContext> {
  constructor(public tasks: QueueTasks, public readonly context: QueueTaskContext<GlobalContext>) {}
}
