import { BadRequestException, Injectable, LoggerService } from '@nestjs/common';
import { BackupsService, JobBackupData, JobService } from '@woodstock/shared';
import {
  QueueSubTask,
  QueueTaskContext,
  QueueTasks,
  QueueTasksInformations,
  QueueTasksService,
} from '@woodstock/shared/tasks';
import { Job } from 'bullmq';

export enum RemoveTaskName {
  REMOVE_REFCNT_POOL_TASK = 'REMOVE_REFCNT_POOL_TASK',
  REMOVE_REFCNT_HOST_TASK = 'REMOVE_REFCNT_HOST_TASK',
  REMOVE_BACKUP_TASK = 'REMOVE_BACKUP_TASK',
}

@Injectable()
export class RemoveService {
  constructor(
    private backupsService: BackupsService,
    private queueTasksService: QueueTasksService,
    private jobService: JobService,
  ) {}

  #createGlobalContext(job: Job<JobBackupData>, hostname: string, backupNumber: number, logger: LoggerService) {
    const globalContext = new QueueTaskContext({}, logger);

    globalContext.commands.set(RemoveTaskName.REMOVE_REFCNT_POOL_TASK, async () => {
      await this.jobService.launchRefcntJob(
        job.id ?? '',
        `${job.prefix}:${job.queueName}`,
        job.data.host,
        job.data.number ?? 0,
        'remove_backup',
      );
    });
    globalContext.commands.set(RemoveTaskName.REMOVE_REFCNT_HOST_TASK, async () => {
      await this.backupsService.removeRefcntOfHost(hostname, backupNumber);
    });
    globalContext.commands.set(RemoveTaskName.REMOVE_BACKUP_TASK, async () => {
      await this.backupsService.removeBackup(hostname, backupNumber);
    });

    return globalContext;
  }

  prepareRemoveTask(job: Job<JobBackupData>, logger: LoggerService) {
    const { host, number } = job.data;
    if (!host || number === undefined) {
      throw new BadRequestException(`Host and backup number should be defined`);
    }

    const task = new QueueTasks('GLOBAL', {})
      .add(new QueueSubTask(RemoveTaskName.REMOVE_REFCNT_HOST_TASK))
      .add(new QueueSubTask(RemoveTaskName.REMOVE_REFCNT_POOL_TASK))
      .add(new QueueSubTask(RemoveTaskName.REMOVE_BACKUP_TASK));

    return new QueueTasksInformations(task, this.#createGlobalContext(job, host, number, logger));
  }

  launchRemoveTask(job: Job<JobBackupData>, informations: QueueTasksInformations<unknown>, signal: AbortSignal) {
    return this.queueTasksService.executeTasksFromJob(job, informations, async () => {
      if (signal.aborted) {
        throw new Error('Aborted task');
      }
    });
  }

  serializeTask(tasks: QueueTasks): object {
    return this.queueTasksService.serializeBackupTask(tasks);
  }

  deserializeTask(data: object): QueueTasks {
    return this.queueTasksService.deserializeBackupTask(data);
  }
}
