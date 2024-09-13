import { BadRequestException, Injectable, LoggerService } from '@nestjs/common';
import { ApplicationConfigService, BackupsService, JobBackupData, JobService } from '@woodstock/shared';
import { WoodstockBackupRemove } from '@woodstock/shared-rs';
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
    private applicationConfig: ApplicationConfigService,
    private queueTasksService: QueueTasksService,
    private jobService: JobService,
  ) {}

  async #createGlobalContext(job: Job<JobBackupData>, hostname: string, backupNumber: number) {
    let remover = await WoodstockBackupRemove.createClient(hostname, backupNumber, this.applicationConfig.context);

    const globalContext = new QueueTaskContext({
      remover,
    });

    globalContext.commands.set(RemoveTaskName.REMOVE_REFCNT_POOL_TASK, async () => {
      await this.jobService.launchRefcntJob(
        job.id ?? '',
        `${job.prefix}:${job.queueName}`,
        job.data.host,
        job.data.number ?? 0,
        'remove_backup',
      );
    });
    globalContext.commands.set(RemoveTaskName.REMOVE_REFCNT_HOST_TASK, async (gc) => {
      await gc.globalContext.remover.removeRefcntOfHost();
    });
    globalContext.commands.set(RemoveTaskName.REMOVE_BACKUP_TASK, async (gc) => {
      await gc.globalContext.remover.removeBackup();
    });

    return globalContext;
  }

  async prepareRemoveTask(job: Job<JobBackupData>) {
    const { host, number } = job.data;
    if (!host || number === undefined) {
      throw new BadRequestException(`Host and backup number should be defined`);
    }

    const task = new QueueTasks('GLOBAL', {})
      .add(new QueueSubTask(RemoveTaskName.REMOVE_REFCNT_HOST_TASK))
      .add(new QueueSubTask(RemoveTaskName.REMOVE_REFCNT_POOL_TASK))
      .add(new QueueSubTask(RemoveTaskName.REMOVE_BACKUP_TASK));

    return new QueueTasksInformations(task, await this.#createGlobalContext(job, host, number));
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
