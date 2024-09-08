import { InjectQueue, Processor, WorkerHost } from '@nestjs/bullmq';
import { BadGatewayException, Logger, NotFoundException } from '@nestjs/common';
import { ApplicationLogger, ResolveService } from '@woodstock/shared';
import { BackupsService, JobBackupData, JobService, QueueName, QUEUE_TASK_FAILED_STATE } from '@woodstock/shared';
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
    private applicationLogger: ApplicationLogger,
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
    const hostAvailable = await this.jobService.hostAvailable(job.data.host);
    if (!shouldBackupHost || !hostAvailable) {
      this.logger.log(
        `STOP: The backup should not be made ${job.data.host} (host available = ${hostAvailable}) - JOB ID = ${job.id}`,
      );
      await job.remove();
      return;
    }

    await this.jobService.using(job, async (signal) => {
      this.logger.debug(`Update the config - JOB ID = ${job.id}`);
      const config = await this.hostConsumerUtilService.updateBackupTaskConfig(job);

      try {
        const backupTask = job.data;
        await this.backupsService.invalidateBackup(backupTask.host);

        this.logger.debug(`Get the next backup number - JOB ID = ${job.id}`);
        if (backupTask.number === undefined) {
          Object.assign(backupTask, await this.jobService.getNextBackup(backupTask.host));
          job.updateData(backupTask);
        }

        return this.applicationLogger.useLogger(backupTask.host, backupTask.number ?? -1, async () => {
          this.logger.debug(`Resolve IP - JOB ID = ${job.id}`);
          if (!backupTask.ip && !backupTask.config?.isLocal) {
            backupTask.ip = await this.resolveService.resolveFromConfig(backupTask.host, config);
            if (!backupTask.ip) {
              throw new BadGatewayException(`Can't find IP for host ${backupTask.host}`);
            }
            job.updateData(backupTask);
          }

          this.logger.debug(`Define the start date - JOB ID = ${job.id}`);
          if (!backupTask.startDate) {
            backupTask.startDate = Date.now();
            job.updateData(backupTask);
          }

          this.logger.debug(`Prepare the backup job - JOB ID = ${job.id}`);
          const informations = await this.backupTasksService.prepareBackupTask(job);

          this.logger.debug(`Launch the backup job - JOB ID = ${job.id}`);
          await this.backupTasksService.launchBackupTask(job, informations, signal);

          this.logger.verbose(
            `PROGRESS: Last backup for job of ${job.data.host} with ${JSON.stringify(
              informations.tasks.progression,
            )} because of ${inspect(this.backupTasksService.serializeBackupTask(informations.tasks), {
              showHidden: false,
              depth: null,
              colors: true,
            })}  - JOB ID = ${job.id}`,
          );

          if (QUEUE_TASK_FAILED_STATE.includes(informations.tasks.state)) {
            throw new LaunchBackupError(`Backup failed for ${job.data.host} with state ${informations.tasks.state}`);
          }
        });
      } catch (err) {
        this.logger.error(`END: Job for ${job.data.host} failed with error: ${err.message} - JOB ID = ${job.id}`, err);
        throw err;
      } finally {
        await this.backupsService.invalidateBackup(job.data.host);

        this.applicationLogger.closeLogger(job.data.host, job.data.number ?? -1);
      }
    });
    this.logger.debug(`END: Of backup of the host ${job.data.host} - JOB ID = ${job.id}`);
  }

  async remove(job: Job<JobBackupData>): Promise<void> {
    this.logger.debug(`START: Remove ${job.data.host} backup number ${job.data.number} - JOB ID = ${job.id}`);
    await this.jobService.using(job, async (signal) => {
      try {
        const backupTask = job.data;

        return this.applicationLogger.useLogger(backupTask.host, backupTask.number ?? -1, async () => {
          if (!backupTask.startDate) {
            backupTask.startDate = Date.now();
            job.updateData(backupTask);
          }

          const informations = this.removeService.prepareRemoveTask(job);
          await this.removeService.launchRemoveTask(job, informations, signal);
        });
      } catch (err) {
        this.logger.error(`END: Job for ${job.data.host} failed with error: ${err.message} - JOB ID = ${job.id}`, err);
        throw err;
      } finally {
        this.logger.log(`[END] Removing backup ${job.data.number} of ${job.data.host} done`);
      }
    });
  }
}
