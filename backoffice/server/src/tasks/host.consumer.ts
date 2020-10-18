import { InjectQueue, Process, Processor } from '@nestjs/bull';
import { BadGatewayException, BadRequestException, Logger } from '@nestjs/common';
import { Job, Queue } from 'bull';
import { lastValueFrom } from 'rxjs';
import { auditTime, map } from 'rxjs/operators';

import { BackupsService } from '../backups/backups.service';
import { BackupLogger } from '../logger/BackupLogger.logger';
import { PingService } from '../network/ping';
import { ResolveService } from '../network/resolve';
import { SchedulerConfigService } from '../scheduler/scheduler-config.service';
import { StatsConsumer } from '../stats/stats.consumer';
import { HostConsumerUtilService } from '../utils/host-consumer-util.service';
import { InternalBackupTask } from './tasks.class';
import { BackupTask } from './tasks.dto';
import { TasksService } from './tasks.service';

const maxBackupTask = parseInt(process.env.MAX_BACKUP_TASK || '') || 1;

@Processor('queue')
export class HostConsumer {
  private logger = new Logger(HostConsumer.name);

  constructor(
    @InjectQueue('queue') private hostsQueue: Queue<BackupTask>,
    private hostConsumerUtilService: HostConsumerUtilService,
    private resolveService: ResolveService,
    private pingService: PingService,
    private tasksService: TasksService,
    private schedulerConfigService: SchedulerConfigService,
    private backupsService: BackupsService,
    private statsConsumer: StatsConsumer,
  ) {}

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

      // Calculate compression size
      await this.statsConsumer.calculateStatsForHost(job, backupTask.number);
    } catch (err) {
      this.logger.error(`END: Job for ${job.data.host} failed with error: ${err.message} - JOB ID = ${job.id}`, err);
    } finally {
      await this.hostConsumerUtilService.unlock(job);
    }
    this.logger.debug(`END: Of backup of the host ${job.data.host} - JOB ID = ${job.id}`);
  }

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
    this.logger.debug(
      `Last backup for the host ${job.data.host} have been made at ${
        timeSinceLastBackup / 3600
      } hours past (should be made after ${schedule.backupPerdiod / 3600} hour)  - JOB ID = ${job.id}`,
    );
    if (schedule.activated && (!lastBackup?.complete || timeSinceLastBackup > schedule.backupPerdiod)) {
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
