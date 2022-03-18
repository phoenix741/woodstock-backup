import { InjectQueue, Process, Processor } from '@nestjs/bull';
import { BadGatewayException, BadRequestException, Logger } from '@nestjs/common';
import {
  BackupLogger,
  BackupsService,
  BackupTask,
  PingService,
  ResolveService,
  SchedulerConfigService,
} from '@woodstock/backoffice-shared';
import { Job, Queue } from 'bull';
import { lastValueFrom } from 'rxjs';
import { auditTime, map } from 'rxjs/operators';
import { HostConsumerUtilService } from '../utils/host-consumer-util.service';
import { InternalBackupTask } from './tasks.class';
import { TasksService } from './tasks.service';

const maxBackupTask = parseInt(process.env.MAX_BACKUP_TASK || '') || 1;

@Processor('queue')
export class HostConsumer {
  private logger = new Logger(HostConsumer.name);

  constructor(
    @InjectQueue('queue') private hostsQueue: Queue<BackupTask>,
    private hostConsumerUtilService: HostConsumerUtilService,
    private resolveService: ResolveService,
    private tasksService: TasksService,
    private backupsService: BackupsService,
    private pingService: PingService,
    private schedulerConfigService: SchedulerConfigService,
  ) {}

  @Process({
    name: 'schedule_host',
    concurrency: 0,
  })
  async schedule(job: Job<BackupTask>): Promise<void> {
    this.logger.log(`START: Test ${job.data.host} for backup - JOB ID = ${job.id}`);

    const config = await this.hostConsumerUtilService.updateBackupTaskConfig(job);
    const schedulerConfig = await this.schedulerConfigService.getScheduler();
    const schedule = Object.assign({}, schedulerConfig.defaultSchedule, config.schedule);

    const lockedJobId = await this.backupsService.isLocked(job.data.host);
    if (lockedJobId && (await (await this.hostsQueue.getJob(lockedJobId))?.isActive())) {
      this.logger.debug(`END: A job (${lockedJobId}) is already running for  ${job.data.host} - JOB ID = ${job.id}`);
      return;
    }

    const lastBackup = await this.backupsService.getLastBackup(job.data.host);

    // If backup is activated, and the last backup is old, we crete a new backup
    const timeSinceLastBackup = (new Date().getTime() - (lastBackup?.startDate || 0)) / 1000;
    const backupPeriod = schedule.backupPeriod || 0;
    this.logger.debug(
      `Last backup for the host ${job.data.host} have been made at ${
        timeSinceLastBackup / 3600
      } hours past (should be made after ${backupPeriod / 3600} hour)  - JOB ID = ${job.id}`,
    );
    if (schedule.activated && (!lastBackup?.complete || timeSinceLastBackup > backupPeriod)) {
      // Check if we can ping
      // Add a more complexe logic, to backup only if not in a blackout period.
      if (await this.pingService.pingFromConfig(job.data.host, config)) {
        // Yes we can, so we backup
        const backupJob = await this.hostsQueue.add('backup', job.data, { removeOnComplete: true });
        this.logger.debug(`END: Host ${job.data.host} Launch backup job ${backupJob.id} - JOB ID = ${job.id}`);
      } else {
        this.logger.debug(`END: Host ${job.data.host} not available on network - JOB ID = ${job.id}`);
      }
    } else {
      this.logger.debug(`END: Host ${job.data.host} will not be backuped - JOB ID = ${job.id}`);
    }

    // Check if some backup should be removed
  }

  @Process({
    name: 'backup',
    concurrency: maxBackupTask,
  })
  async launchBackup(job: Job<BackupTask>): Promise<void> {
    this.logger.log(`START: Launch the backup of the host ${job.data.host} - JOB ID = ${job.id}`);

    const config = await this.hostConsumerUtilService.updateBackupTaskConfig(job);

    await this.hostConsumerUtilService.lock(job);

    try {
      const backupTask = job.data;
      if (backupTask.number === undefined) {
        const lastBackup = await this.backupsService.getLastBackup(job.data.host);
        if (lastBackup?.complete) {
          backupTask.number = lastBackup.number + 1;
          backupTask.previousNumber = lastBackup.number;
        } else {
          backupTask.number = lastBackup?.number || 0;
        }
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
      this.tasksService.prepareBackup(task);

      await lastValueFrom(
        this.tasksService.launchBackup(backupLogger, task).pipe(
          auditTime(5000), // FIXME: Conf
          map(async (task) => {
            job.update(task);
            job.progress(task.progression?.percent);
            await this.backupsService.addOrReplaceBackup(job.data.host, task.toBackup());
            return task;
          }),
        ),
      );

      job.update(task);
      job.progress(task.progression?.percent);

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

  @Process('remove_backup')
  async remove(job: Job<BackupTask>): Promise<void> {
    this.logger.debug(`START: Remove ${job.data.host} backup number ${job.data.number} - JOB ID = ${job.id}`);
    if (!job.data.host || job.data.number === undefined) {
      throw new BadRequestException(`Host and backup number should be defined`);
    }

    await this.hostConsumerUtilService.lock(job);
    try {
      await this.backupsService.removeBackup(job.data.host, job.data.number);
      // FIXME: Remove backup files
    } finally {
      await this.hostConsumerUtilService.unlock(job);
    }
  }
}
