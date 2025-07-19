import { BadGatewayException, Injectable, Logger, NotFoundException } from '@nestjs/common';
import {
  BackupQueueData,
  BackupsService,
  JobBackupData,
  JobCleanupData,
  JobFsckData,
  JobRemoveData,
  JobRestoreData,
  JobService,
  PingService,
  QueueTasksService,
} from '@woodstock/shared';
import { SandboxedJob } from 'bullmq';
import { BackupMachineService } from '../backups/backup-machine.service.js';
import { LaunchBackupError } from '../backups/backup.error.js';
import { RemoveMachineService } from '../backups/remove-machine.service.js';
import { RestoreMachineService } from '../backups/restore-machine.service.js';
import { CleanupMachineService } from '../pool/cleanup-machine.service.js';
import { FsckMachineService } from '../pool/fsck-machine.service.js';
import { HostConsumerUtilService } from '../utils/host-consumer-util.service.js';
import { BackupLogger } from '../backup.logger.js';

@Injectable()
export class HostConsumer {
  #logger = new Logger(HostConsumer.name);

  constructor(
    private readonly logger: BackupLogger,
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
  ) {}

  async process(job: SandboxedJob<BackupQueueData>): Promise<void> {
    switch (job.name) {
      case 'backup':
        await this.#launchBackup(job as SandboxedJob<JobBackupData>);
        break;
      case 'remove_backup':
        await this.#remove(job as SandboxedJob<JobRemoveData>);
        break;
      case 'restore':
        await this.#restore(job as SandboxedJob<JobRestoreData>);
        break;
      case 'cleanup_refcnt':
        await this.#processCleanup(job as SandboxedJob<JobCleanupData>);
        break;
      case 'fsck':
        await this.#processFsck(job as SandboxedJob<JobFsckData>);
        break;
      default:
        throw new NotFoundException(`Unknown job name ${job.name}`);
    }
  }

  async #launchBackup(job: SandboxedJob<JobBackupData>): Promise<void> {
    this.#logger.log(`START: Launch the restore of the host ${job.data.host} - JOB ID = ${job.id}`);
    const shouldBackupHost = await this.jobService.shouldBackupHost(job.data.host, job.id, job.data.force);
    const hostAvailable = await this.jobService.hostAvailable(job.data.host);
    if (!shouldBackupHost || !hostAvailable) {
      this.#logger.log(
        `STOP: The backup should not be made ${job.data.host} (host available = ${hostAvailable}) - JOB ID = ${job.id}`,
      );
      return;
    }

    this.#logger.debug(`Update the config - JOB ID = ${job.id}`);
    const config = await this.hostConsumerUtilService.updateBackupTaskConfig(job);

    try {
      const backupTask = job.data;

      await this.backupsService.invalidateBackup(backupTask.host);

      this.#logger.debug(`Get the next backup number - JOB ID = ${job.id}`);
      if (backupTask.number === undefined) {
        Object.assign(backupTask, await this.jobService.getNextBackup(backupTask.host));
        job.updateData(backupTask);
        this.logger.updateLogger(
          job.id,
          job.name,
          (job.data as JobBackupData | JobRestoreData)?.host,
          (job.data as JobBackupData | JobRestoreData)?.number,
        );
      }

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

      const states$ = this.backupService.execute(backupTask.host, backupTask.ip, backupTask.number ?? 0);
      const lastProgress = await this.queueTaskService.processJobData(job, states$);
      if (lastProgress.errorState) {
        throw new LaunchBackupError(`Backup failed for ${job.data.host} with state ${lastProgress.errorMessage}`);
      }
    } catch (err) {
      this.#logger.error(`END: Job for ${job.data.host} failed with error: ${err.message} - JOB ID = ${job.id}`, err);
      throw err;
    } finally {
      await this.backupsService.invalidateBackup(job.data.host);
    }
    this.#logger.debug(`END: Of backup of the host ${job.data.host} - JOB ID = ${job.id}`);
  }

  async #restore(job: SandboxedJob<JobRestoreData>): Promise<void> {
    this.#logger.log(`START: Launch the restore of the host ${job.data.host} - JOB ID = ${job.id}`);
    const hostAvailable = await this.jobService.hostAvailable(job.data.host);
    if (!hostAvailable) {
      this.#logger.log(
        `STOP: The restore can't be made ${job.data.host} (host available = ${hostAvailable}) - JOB ID = ${job.id}`,
      );
      return;
    }

    this.#logger.debug(`Update the config - JOB ID = ${job.id}`);
    const config = await this.hostConsumerUtilService.updateBackupTaskConfig(job);

    try {
      const backupTask = job.data;

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

      const states$ = this.restoreService.execute(
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
    } catch (err) {
      this.#logger.error(`END: Job for ${job.data.host} failed with error: ${err.message} - JOB ID = ${job.id}`, err);
      throw err;
    }
    this.#logger.debug(`END: Of backup of the host ${job.data.host} - JOB ID = ${job.id}`);
  }

  async #remove(job: SandboxedJob<JobRemoveData>): Promise<void> {
    this.#logger.debug(`START: Remove ${job.data.host} backup number ${job.data.number} - JOB ID = ${job.id}`);

    try {
      const backupTask = job.data;

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
    } catch (err) {
      this.#logger.error(`END: Job for ${job.data.host} failed with error: ${err.message} - JOB ID = ${job.id}`, err);
      throw err;
    } finally {
      this.#logger.log(`[END] Removing backup ${job.data.number} of ${job.data.host} done`);
    }
  }

  async #processFsck(job: SandboxedJob<JobFsckData>): Promise<void> {
    this.#logger.log(`START: Processing job FSCK ${job.id}`);

    const jobId = job.id ?? 'unknown_jobid';
    try {
      const backupTask = job.data;

      const states$ = this.fsckService.execute(backupTask.dryRun, backupTask.verifyChunks);
      const lastProgress = await this.queueTaskService.processJobData(job, states$);
      if (lastProgress.errorState) {
        throw new LaunchBackupError(`Pool check failed with error ${lastProgress.errorMessage}`);
      }
    } catch (err) {
      this.#logger.error(`END: Fsck Job failed with error: ${err.message} - JOB ID = ${job.id}`, err);
      throw err;
    }
    this.#logger.debug(`END: Pool check - JOB ID = ${jobId}`);
  }

  async #processCleanup(job: SandboxedJob<JobCleanupData>): Promise<void> {
    this.#logger.log(`START: Processing job CLEANUP ${job.id}`);

    const jobId = job.id ?? 'unknown_jobid';
    try {
      const taskData = job.data;

      const states$ = this.cleanupService.execute(taskData.target);
      const lastProgress = await this.queueTaskService.processJobData(job, states$);
      if (lastProgress.errorState) {
        throw new LaunchBackupError(`Pool check failed with error ${lastProgress.errorMessage}`);
      }
    } catch (err) {
      this.#logger.error(`END: Fsck Job failed with error: ${err.message} - JOB ID = ${job.id}`, err);
      throw err;
    }
    this.#logger.debug(`END: Pool check - JOB ID = ${jobId}`);
  }
}
