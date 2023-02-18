import { BadRequestException, Injectable, Logger, LoggerService } from '@nestjs/common';
import {
  ApplicationConfigService,
  BackupsService,
  JobBackupData,
  JobService,
  RefCntService,
  ReferenceCount,
} from '@woodstock/shared';
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
  #logger = new Logger(RemoveService.name);

  constructor(
    private applicationConfig: ApplicationConfigService,
    private backupsService: BackupsService,
    private refcntService: RefCntService,
    private queueTasksService: QueueTasksService,
    private jobService: JobService,
  ) {}

  #createGlobalContext(job: Job<JobBackupData>, hostname: string, backupNumber: number, logger: LoggerService) {
    const refcnt = new ReferenceCount(
      this.backupsService.getHostDirectory(hostname),
      this.backupsService.getDestinationDirectory(hostname, backupNumber),
      this.applicationConfig.poolPath,
    );

    const globalContext = new QueueTaskContext(refcnt, logger);

    globalContext.commands.set(RemoveTaskName.REMOVE_REFCNT_POOL_TASK, async (gc) => {
      await this.jobService.launchRefcntJob(
        job.id || '',
        `${job.prefix}:${job.queueName}`,
        hostname,
        backupNumber,
        'remove_backup',
      );
    });
    globalContext.commands.set(RemoveTaskName.REMOVE_REFCNT_HOST_TASK, async (gc) => {
      await this.refcntService.removeBackupRefcntTo(refcnt.hostPath, refcnt.backupPath);
    });
    globalContext.commands.set(RemoveTaskName.REMOVE_BACKUP_TASK, async (gc) => {
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
      .add(new QueueSubTask(RemoveTaskName.REMOVE_REFCNT_HOST_TASK, {}))
      .add(new QueueSubTask(RemoveTaskName.REMOVE_BACKUP_TASK, {}));

    return new QueueTasksInformations(task, this.#createGlobalContext(job, host, number, logger));
  }

  launchRemoveTask(informations: QueueTasksInformations<ReferenceCount>) {
    const { context, tasks } = informations;

    return this.queueTasksService.executeTasks(tasks, context);
  }

  serializeTask(tasks: QueueTasks): object {
    return this.queueTasksService.serializeBackupTask(tasks);
  }

  deserializeTask(data: object): QueueTasks {
    return this.queueTasksService.deserializeBackupTask(data);
  }
}
