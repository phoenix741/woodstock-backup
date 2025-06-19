import { Processor, WorkerHost } from '@nestjs/bullmq';
import { BadGatewayException, Logger, NotFoundException } from '@nestjs/common';
import {
  ApplicationLogger,
  BackupQueueData,
  BackupsService,
  JobBackupData,
  JobCleanupData,
  JobFsckData,
  JobRemoveData,
  JobRestoreData,
  JobService,
  PingService,
  QueueName,
  QueueTasksService,
} from '@woodstock/shared';
import { Job } from 'bullmq';
import { BackupMachineService } from '../backups/backup-machine.service.js';
import { LaunchBackupError } from '../backups/backup.error.js';
import { RemoveMachineService } from '../backups/remove-machine.service.js';
import { RestoreMachineService } from '../backups/restore-machine.service.js';
import { CleanupMachineService } from '../pool/cleanup-machine.service.js';
import { FsckMachineService } from '../pool/fsck-machine.service.js';
import { HostConsumerUtilService } from '../utils/host-consumer-util.service.js';

const maxBackupTask = parseInt(process.env.MAX_BACKUP_TASK || '') || 2;

@Processor(QueueName.BACKUP_QUEUE, {
  concurrency: maxBackupTask,
  removeOnComplete: {
    // TODO: Configuration
    age: 60 * 60 * 24 * 7,
  },
  removeOnFail: {
    age: 60 * 60 * 24 * 7 * 2,
  },
})
export class HostConsumer extends WorkerHost {
  #logger = new Logger(HostConsumer.name);

  constructor(
    private applicationLogger: ApplicationLogger,
    private hostConsumerUtilService: HostConsumerUtilService,
    private pingService: PingService,
    private backupsService: BackupsService,
    private jobService: JobService,
    private backupService: BackupMachineService,
    private restoreService: RestoreMachineService,
    private removeService: RemoveMachineService,
    private fsckService: FsckMachineService,
    private cleanupService: CleanupMachineService,
    private queueTaskService: QueueTasksService,
  ) {
    super();
  }

  async process(job: Job<BackupQueueData>): Promise<void> {
    switch (job.name) {
      case 'backup':
        await this.#launchBackup(job as Job<JobBackupData>);
        break;
      case 'remove_backup':
        await this.#remove(job as Job<JobRemoveData>);
        break;
      case 'restore':
        await this.#restore(job as Job<JobRestoreData>);
        break;
      case 'cleanup_refcnt':
        await this.#processCleanup(job as Job<JobCleanupData>);
        break;
      case 'fsck':
        await this.#processFsck(job as Job<JobFsckData>);
        break;
      default:
        throw new NotFoundException(`Unknown job name ${job.name}`);
    }
  }

  async #launchBackup(job: Job<JobBackupData>): Promise<void> {
    this.#logger.log(`START: Launch the restore of the host ${job.data.host} - JOB ID = ${job.id}`);
    const shouldBackupHost = await this.jobService.shouldBackupHost(job.data.host, job.id, job.data.force);
    const hostAvailable = await this.jobService.hostAvailable(job.data.host);
    if (!shouldBackupHost || !hostAvailable) {
      this.#logger.log(
        `STOP: The backup should not be made ${job.data.host} (host available = ${hostAvailable}) - JOB ID = ${job.id}`,
      );
      await job.remove();
      return;
    }

    const jobId = job.id ?? 'unknown_jobid';
    this.#logger.debug(`Update the config - JOB ID = ${job.id}`);
    const config = await this.hostConsumerUtilService.updateBackupTaskConfig(job);

    try {
      const backupTask = job.data;

      await this.backupsService.invalidateBackup(backupTask.host);

      this.#logger.debug(`Get the next backup number - JOB ID = ${job.id}`);
      if (backupTask.number === undefined) {
        Object.assign(backupTask, await this.jobService.getNextBackup(backupTask.host));
        job.updateData(backupTask);
      }

      await this.applicationLogger.useLogger(
        { jobId, hostname: backupTask.host, backupNumber: backupTask.number ?? -1, operation: 'backup' },
        async () => {
          this.#logger.debug(`Resolve IP - JOB ID = ${job.id}`);
          if (!backupTask.ip) {
            backupTask.ip = await this.pingService.pingFromConfig(backupTask.host, config);
            if (!backupTask.ip) {
              throw new BadGatewayException(`Can't find IP for host ${backupTask.host}`);
            }
            job.updateData(backupTask);
          }

          this.#logger.debug(`Define the start date - JOB ID = ${job.id}`);
          if (!backupTask.startDate) {
            backupTask.startDate = Date.now();
            job.updateData(backupTask);
          }

          const states$ = await this.backupService.execute(backupTask.host, backupTask.ip, backupTask.number ?? 0);
          const lastProgress = await this.queueTaskService.processJobData(job, states$);
          if (lastProgress.errorState) {
            throw new LaunchBackupError(`Backup failed for ${job.data.host} with state ${lastProgress.errorMessage}`);
          }
        },
      );
    } catch (err) {
      this.#logger.error(`END: Job for ${job.data.host} failed with error: ${err.message} - JOB ID = ${job.id}`, err);
      throw err;
    } finally {
      await this.backupsService.invalidateBackup(job.data.host);

      this.applicationLogger.closeLogger(jobId);
    }
    this.#logger.debug(`END: Of backup of the host ${job.data.host} - JOB ID = ${job.id}`);
  }

  async #restore(job: Job<JobRestoreData>): Promise<void> {
    this.#logger.log(`START: Launch the restore of the host ${job.data.host} - JOB ID = ${job.id}`);
    const hostAvailable = await this.jobService.hostAvailable(job.data.host);
    if (!hostAvailable) {
      this.#logger.log(
        `STOP: The restore can't be made ${job.data.host} (host available = ${hostAvailable}) - JOB ID = ${job.id}`,
      );
      await job.remove();
      return;
    }

    const jobId = job.id ?? 'unknown_jobid';
    this.#logger.debug(`Update the config - JOB ID = ${job.id}`);
    const config = await this.hostConsumerUtilService.updateBackupTaskConfig(job);

    try {
      const backupTask = job.data;

      await this.applicationLogger.useLogger(
        { jobId: jobId, hostname: backupTask.host, backupNumber: backupTask.number ?? -1, operation: 'restore' },
        async () => {
          this.#logger.debug(`Resolve IP - JOB ID = ${job.id}`);
          if (!backupTask.ip) {
            const ip = await this.pingService.pingFromConfig(backupTask.host, config);
            if (!ip) {
              throw new BadGatewayException(`Can't find IP for host ${backupTask.host}`);
            }
            backupTask.ip = ip;

            job.updateData(backupTask);
          }

          this.#logger.debug(`Define the start date - JOB ID = ${job.id}`);
          if (!backupTask.startDate) {
            backupTask.startDate = Date.now();
            job.updateData(backupTask);
          }

          const states$ = await this.restoreService.execute(
            backupTask.host,
            backupTask.ip,
            backupTask.number ?? 0,
            backupTask.destinationDirectory,
            backupTask.files,
          );
          const lastProgress = await this.queueTaskService.processJobData(job, states$);
          if (lastProgress.errorState) {
            throw new LaunchBackupError(`Restore failed for ${job.data.host} with state ${lastProgress.errorMessage}`);
          }
        },
      );
    } catch (err) {
      this.#logger.error(`END: Job for ${job.data.host} failed with error: ${err.message} - JOB ID = ${job.id}`, err);
      throw err;
    } finally {
      this.applicationLogger.closeLogger(jobId);
    }
    this.#logger.debug(`END: Of backup of the host ${job.data.host} - JOB ID = ${job.id}`);
  }

  async #remove(job: Job<JobRemoveData>): Promise<void> {
    this.#logger.debug(`START: Remove ${job.data.host} backup number ${job.data.number} - JOB ID = ${job.id}`);

    const jobId = job.id ?? 'unknown_jobid';
    try {
      const backupTask = job.data;

      await this.applicationLogger.useLogger(
        { jobId, hostname: backupTask.host, backupNumber: backupTask.number ?? -1, operation: 'remove' },
        async () => {
          if (!backupTask.startDate) {
            backupTask.startDate = Date.now();
            job.updateData(backupTask);
          }

          const states$ = await this.removeService.execute(backupTask.host, backupTask.number ?? 0);
          const lastProgress = await this.queueTaskService.processJobData(job, states$);
          if (lastProgress.errorState) {
            throw new LaunchBackupError(
              `Remove operation failed for ${job.data.host} with state ${lastProgress.errorMessage}`,
            );
          }
        },
      );
    } catch (err) {
      this.#logger.error(`END: Job for ${job.data.host} failed with error: ${err.message} - JOB ID = ${job.id}`, err);
      throw err;
    } finally {
      this.applicationLogger.closeLogger(jobId);
      this.#logger.log(`[END] Removing backup ${job.data.number} of ${job.data.host} done`);
    }
  }

  async #processFsck(job: Job<JobFsckData>): Promise<void> {
    this.#logger.log(`START: Processing job FSCK ${job.id}`);

    const jobId = job.id ?? 'unknown_jobid';
    try {
      const backupTask = job.data;

      await this.applicationLogger.useLogger({ jobId, operation: 'refcnt' }, async () => {
        const states$ = this.fsckService.execute(backupTask.dryRun, backupTask.verifyChunks);
        const lastProgress = await this.queueTaskService.processJobData(job, states$);
        if (lastProgress.errorState) {
          throw new LaunchBackupError(`Pool check failed with error ${lastProgress.errorMessage}`);
        }
      });
    } catch (err) {
      this.#logger.error(`END: Fsck Job failed with error: ${err.message} - JOB ID = ${job.id}`, err);
      throw err;
    } finally {
      this.applicationLogger.closeLogger(jobId);
    }
    this.#logger.debug(`END: Pool check - JOB ID = ${jobId}`);
  }

  async #processCleanup(job: Job<JobCleanupData>): Promise<void> {
    this.#logger.log(`START: Processing job CLEANUP ${job.id}`);

    const jobId = job.id ?? 'unknown_jobid';
    try {
      const taskData = job.data;

      await this.applicationLogger.useLogger({ jobId, operation: 'refcnt' }, async () => {
        const states$ = this.cleanupService.execute(taskData.target);
        const lastProgress = await this.queueTaskService.processJobData(job, states$);
        if (lastProgress.errorState) {
          throw new LaunchBackupError(`Pool check failed with error ${lastProgress.errorMessage}`);
        }
      });
    } catch (err) {
      this.#logger.error(`END: Fsck Job failed with error: ${err.message} - JOB ID = ${job.id}`, err);
      throw err;
    } finally {
      this.applicationLogger.closeLogger(jobId);
    }
    this.#logger.debug(`END: Pool check - JOB ID = ${jobId}`);
  }
}
