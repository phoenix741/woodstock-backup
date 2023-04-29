import { InjectQueue, Processor, WorkerHost } from '@nestjs/bullmq';
import { BadGatewayException, Logger, NotFoundException } from '@nestjs/common';
import { BackupLogger, BackupsService, JobService, QueueName, ResolveService } from '@woodstock/shared';
import { JobBackupData } from '@woodstock/shared/backuping/backuping.model.js';
import { QUEUE_TASK_FAILED_STATE } from '@woodstock/shared/tasks/queue-tasks.model.js';
import { Job, Queue } from 'bullmq';
import { inspect } from 'util';
import { LaunchBackupError } from '../backups/backup.error.js';
import { HostConsumerUtilService } from '../utils/host-consumer-util.service.js';
import { BackupTasksService } from './backup-tasks.service.js';
import { RemoveService } from './remove.service.js';

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
  private logger = new Logger(HostConsumer.name);

  constructor(
    @InjectQueue(QueueName.BACKUP_QUEUE) private hostsQueue: Queue<JobBackupData>,
    private hostConsumerUtilService: HostConsumerUtilService,
    private resolveService: ResolveService,
    private backupsService: BackupsService,
    private removeService: RemoveService,
    private jobService: JobService,
    private backupTasksService: BackupTasksService,
  ) {
    super();
  }

  async process(job: Job<JobBackupData>): Promise<void> {
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

  async launchBackup(job: Job<JobBackupData>): Promise<void> {
    this.logger.log(`START: Launch the backup of the host ${job.data.host} - JOB ID = ${job.id}`);
    const shouldBackupHost = await this.jobService.shouldBackupHost(job.data.host, job.id, job.data.force);
    if (!shouldBackupHost) {
      this.logger.log(`STOP: The backup should not be made ${job.data.host} - JOB ID = ${job.id}`);
      await job.remove();
      return;
    }

    await this.jobService.using(job, async (signal) => {
      this.logger.debug(`Update the config - JOB ID = ${job.id}`);
      const config = await this.hostConsumerUtilService.updateBackupTaskConfig(job);

      try {
        const backupTask = job.data;
        this.logger.debug(`Get the next backup number - JOB ID = ${job.id}`);
        if (backupTask.number === undefined) {
          Object.assign(backupTask, await this.jobService.getNextBackup(backupTask.host));
          job.update(backupTask);
        }

        this.logger.debug(`Resolve IP - JOB ID = ${job.id}`);
        if (!backupTask.ip && !backupTask.config?.isLocal) {
          backupTask.ip = await this.resolveService.resolveFromConfig(backupTask.host, config);
          if (!backupTask.ip) {
            throw new BadGatewayException(`Can't find IP for host ${backupTask.host}`);
          }
          job.update(backupTask);
        }

        this.logger.debug(`Define the start date - JOB ID = ${job.id}`);
        if (!backupTask.startDate) {
          backupTask.startDate = Date.now();
          job.update(backupTask);
        }

        // Set the logger
        const backupLogger = new BackupLogger(this.backupsService, job.data.host, job.data.number);
        const clientLogger = new BackupLogger(this.backupsService, job.data.host, job.data.number, true);

        this.logger.debug(`Prepare the backup job - JOB ID = ${job.id}`);
        const informations = this.backupTasksService.prepareBackupTask(job, clientLogger, backupLogger);

        this.logger.debug(`Launch the backup job - JOB ID = ${job.id}`);
        await this.backupTasksService.launchBackupTask(job, informations, signal);

        this.logger.verbose(
          `PROGRESS: Last backup for job of ${job.data.host} with ${JSON.stringify(
            this.backupTasksService.toBackup(informations),
          )} because of ${inspect(this.backupTasksService.serializeBackupTask(informations.tasks), {
            showHidden: false,
            depth: null,
            colors: true,
          })}  - JOB ID = ${job.id}`,
        );
        await this.backupsService.addOrReplaceBackup(job.data.host, this.backupTasksService.toBackup(informations));

        if (QUEUE_TASK_FAILED_STATE.includes(informations.tasks.state)) {
          throw new LaunchBackupError(`Backup failed for ${job.data.host} with state ${informations.tasks.state}`);
        }
      } catch (err) {
        this.logger.error(`END: Job for ${job.data.host} failed with error: ${err.message} - JOB ID = ${job.id}`, err);
        throw err;
      } finally {
        // Check if the previous backup is incomplete, we can remove it
        const mayBeIncompleteBackup = await this.backupsService.getPreviousBackup(job.data.host, job.data.number || -1);
        if (mayBeIncompleteBackup && !mayBeIncompleteBackup.complete) {
          await this.hostsQueue.add('remove_backup', { host: job.data.host, number: mayBeIncompleteBackup.number });
        }
      }
    });
    this.logger.debug(`END: Of backup of the host ${job.data.host} - JOB ID = ${job.id}`);
  }

  async remove(job: Job<JobBackupData>): Promise<void> {
    this.logger.debug(`START: Remove ${job.data.host} backup number ${job.data.number} - JOB ID = ${job.id}`);
    await this.jobService.using(job, async (signal) => {
      const removeLogger = new BackupLogger(this.backupsService, job.data.host, job.data.number);
      try {
        const backupTask = job.data;

        if (!backupTask.startDate) {
          backupTask.startDate = Date.now();
          job.update(backupTask);
        }

        const informations = this.removeService.prepareRemoveTask(job, removeLogger);
        await this.removeService.launchRemoveTask(job, informations, signal);
      } catch (err) {
        this.logger.error(`END: Job for ${job.data.host} failed with error: ${err.message} - JOB ID = ${job.id}`, err);
        throw err;
      } finally {
        this.logger.log(`[END] Removing backup ${job.data.number} of ${job.data.host} done`);
      }
    });
  }
}
