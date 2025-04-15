import { BadRequestException, Injectable, InternalServerErrorException } from '@nestjs/common';
import { HostConfiguration, JobBackupData } from '@woodstock/shared';
import { generateContext, WoodstockBackupRestore } from '@woodstock/shared-rs';
import {
  QueueSubTask,
  QueueTaskContext,
  QueueTaskProgression,
  QueueTasks,
  QueueTasksInformations,
  QueueTasksService,
} from '@woodstock/shared/tasks';
import { Job } from 'bullmq';
import { Observable } from 'rxjs';

export enum RestoreTaskName {
  RESTORE_TASK_NAME_AUTHENTICATE = 'RESTORE_TASK_NAME_AUTHENTICATE',
  RESTORE_TASK_NAME_PREPARE = 'RESTORE_TASK_NAME_PREPARE',
  RESTORE_TASK_NAME_RESTORE = 'RESTORE_TASK_NAME_RESTORE',
  RESTORE_TASK_NAME_CLOSE = 'RESTORE_TASK_NAME_CLOSE',
}

export class RestoreContext {
  host: string;
  config: HostConfiguration;
  number: number;
  ip: string;
  destinationDirectory?: string;

  remover: WoodstockBackupRestore;

  constructor(jobData: JobBackupData, remover: WoodstockBackupRestore) {
    if (!jobData.config || jobData.number === undefined || !jobData.ip) {
      throw new BadRequestException(`Initialisation of backup failed.`);
    }

    this.host = jobData.host;
    this.config = jobData.config;
    this.number = jobData.number;
    this.ip = jobData.ip;
    this.destinationDirectory = jobData.destinationDirectory;
    this.remover = remover;
  }
}

@Injectable()
export class RestoreService {
  constructor(private queueTasksService: QueueTasksService) {}

  async #createGlobalContext(job: Job<JobBackupData>, hostname: string, ip: string, backupNumber: number) {
    const context = generateContext({
      username: undefined,
    });
    const remover = await WoodstockBackupRestore.createClient(hostname, ip, backupNumber, context);

    const globalContext = new QueueTaskContext<RestoreContext>(new RestoreContext(job.data, remover));

    globalContext.commands.set(RestoreTaskName.RESTORE_TASK_NAME_AUTHENTICATE, async (gc) => {
      if (!gc.globalContext.config?.password) {
        throw new InternalServerErrorException('No password provided');
      }

      await gc.globalContext.remover.authenticate(gc.globalContext.config?.password);
    });
    globalContext.commands.set(RestoreTaskName.RESTORE_TASK_NAME_PREPARE, async (gc, lc) => {
      if (!lc.sharePath || !lc.selection) {
        throw new InternalServerErrorException('No sharePath or selection provided');
      }

      await gc.globalContext.remover.prepareRestauration(lc.sharePath, lc.selection);
    });
    globalContext.commands.set(RestoreTaskName.RESTORE_TASK_NAME_RESTORE, (gc, lc) => {
      return new Observable((observer) => {
        if (!lc.sharePath || !lc.selection) {
          throw new InternalServerErrorException('No sharePath or selection provided');
        }

        gc.globalContext.remover.restore(
          lc.sharePath,
          gc.globalContext.destinationDirectory ?? '',
          lc.selection,
          (progression) => {
            if (progression.progress) {
              observer.next(
                new QueueTaskProgression({
                  progressCurrent: progression.progress.progressCurrent,
                  progressMax: progression.progress.progressMax,
                  fileSize: progression.progress.fileSize,
                  fileCount: progression.progress.fileCount,
                }),
              );
            }
            if (progression.error) {
              observer.error(progression.error);
            }
            if (progression.complete) {
              observer.complete();
            }
          },
        );
      });
    });
    globalContext.commands.set(RestoreTaskName.RESTORE_TASK_NAME_CLOSE, async (gc) => {
      await gc.globalContext.remover.close();
    });

    return globalContext;
  }

  async prepareRestoreTask(job: Job<JobBackupData>) {
    const { host, number, ip } = job.data;
    if (!host || !ip || number === undefined) {
      throw new BadRequestException(`Host, IP, and backup number should be defined`);
    }

    const task = new QueueTasks('GLOBAL', {}).add(new QueueSubTask(RestoreTaskName.RESTORE_TASK_NAME_AUTHENTICATE));

    for (const share of job.data.files ?? []) {
      task
        .add(
          new QueueSubTask(RestoreTaskName.RESTORE_TASK_NAME_PREPARE, {
            sharePath: share.share,
            selection: share.selection,
          }),
        )
        .add(
          new QueueSubTask(RestoreTaskName.RESTORE_TASK_NAME_RESTORE, {
            sharePath: share.share,
            selection: share.selection,
          }),
        );
    }

    task.add(new QueueSubTask(RestoreTaskName.RESTORE_TASK_NAME_CLOSE));

    return new QueueTasksInformations(task, await this.#createGlobalContext(job, host, ip, number));
  }

  launchRestoreTask(
    job: Job<JobBackupData>,
    informations: QueueTasksInformations<RestoreContext>,
    signal: AbortSignal,
  ) {
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
