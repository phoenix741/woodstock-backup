import { Processor, WorkerHost } from '@nestjs/bullmq';
import { BadGatewayException, BadRequestException, Logger, NotFoundException } from '@nestjs/common';
import { BackupLogger, BackupsService, BackupTask, JobService, ResolveService } from '@woodstock/shared';
import { Job } from 'bullmq';
import { lastValueFrom } from 'rxjs';
import { auditTime, map } from 'rxjs/operators';
import { HostConsumerUtilService } from '../utils/host-consumer-util.service.js';
import { RemoveService } from './remove.service.js';
import { InternalBackupTask } from './tasks.class.js';
import { TasksService } from './tasks.service.js';

const maxBackupTask = parseInt(process.env.MAX_BACKUP_TASK || '') || 1;

@Processor('queue', { concurrency: maxBackupTask })
export class HostConsumer extends WorkerHost {
  private logger = new Logger(HostConsumer.name);

  constructor(
    private hostConsumerUtilService: HostConsumerUtilService,
    private resolveService: ResolveService,
    private tasksService: TasksService,
    private backupsService: BackupsService,
    private removeService: RemoveService,
    private jobService: JobService,
  ) {
    super();
  }

  async process(job: Job<BackupTask>): Promise<void> {
    switch (job.name) {
      case 'backup':
        await this.launchBackup(job);
        break;
      case 'remove_backup':
        await this.remove(job);
        break;
      default:
        throw new NotFoundException(`Unknown job name ${job.name}`);
    }
  }

  async launchBackup(job: Job<BackupTask>): Promise<void> {
    this.logger.log(`START: Launch the backup of the host ${job.data.host} - JOB ID = ${job.id}`);
    const shouldBackupHost = await this.jobService.shouldBackupHost(job.data.host, job.id, job.data.force);
    if (!shouldBackupHost) {
      this.logger.log(`STOP: Launch the backup of the host ${job.data.host} - JOB ID = ${job.id}`);
      return;
    }

    await this.hostConsumerUtilService.lock(job);
    const config = await this.hostConsumerUtilService.updateBackupTaskConfig(job);

    try {
      const backupTask = job.data;
      if (backupTask.number === undefined) {
        Object.assign(backupTask, await this.jobService.getLastBackup(backupTask.host));
        job.update(backupTask);
      }

      if (!backupTask.ip && !backupTask.config?.isLocal) {
        backupTask.ip = await this.resolveService.resolveFromConfig(backupTask.host, config);
        if (!backupTask.ip) {
          throw new BadGatewayException(`Can't find IP for host ${backupTask.host}`);
        }
        job.update(backupTask);
      }

      // Set the logger
      const backupLogger = new BackupLogger(this.backupsService, job.data.host, job.data.number);

      const task = new InternalBackupTask(backupTask);
      this.tasksService.prepareBackup(job, task);

      await lastValueFrom(
        this.tasksService.launchBackup(backupLogger, task).pipe(
          auditTime(5000), // TODO: Conf
          map(async (task) => {
            job.update(task);
            job.updateProgress(task.progression?.percent);
            await this.backupsService.addOrReplaceBackup(job.data.host, task.toBackup());
            return task;
          }),
        ),
      );

      job.update(task);
      job.updateProgress(task.progression?.percent);

      this.logger.verbose(
        `PROGRESS: Last backup for job of ${job.data.host} with ${JSON.stringify(
          task.toBackup(),
        )} because of ${JSON.stringify(task.subtasks)}  - JOB ID = ${job.id}`,
      );
      await this.backupsService.addOrReplaceBackup(job.data.host, task.toBackup());
    } catch (err) {
      this.logger.error(`END: Job for ${job.data.host} failed with error: ${err.message} - JOB ID = ${job.id}`, err);
      throw err;
    } finally {
      await this.hostConsumerUtilService.unlock(job);
    }
    this.logger.debug(`END: Of backup of the host ${job.data.host} - JOB ID = ${job.id}`);
  }

  async remove(job: Job<BackupTask>): Promise<void> {
    this.logger.debug(`START: Remove ${job.data.host} backup number ${job.data.number} - JOB ID = ${job.id}`);
    if (!job.data.host || job.data.number === undefined) {
      throw new BadRequestException(`Host and backup number should be defined`);
    }

    await this.hostConsumerUtilService.lock(job);
    try {
      await this.jobService.launchRefcntJob(
        job.id || '',
        `${job.prefix}:${job.queueName}`,
        job.data.host,
        job.data.number,
        'remove_backup',
      );

      await this.removeService.remove(job.data.host, job.data.number);
    } catch (err) {
      this.logger.error(`END: Job for ${job.data.host} failed with error: ${err.message} - JOB ID = ${job.id}`, err);
      throw err;
    } finally {
      await this.hostConsumerUtilService.unlock(job);
    }
  }
}
