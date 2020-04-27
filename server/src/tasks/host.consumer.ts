import { InjectQueue, Process, Processor } from '@nestjs/bull';
import { BadGatewayException, BadRequestException, Logger, NotFoundException } from '@nestjs/common';
import { Job, Queue } from 'bull';
import { auditTime, map } from 'rxjs/operators';

import { BackupsService } from '../backups/backups.service';
import { ApplicationConfigService } from '../config/application-config.service';
import { HostsService } from '../hosts/hosts.service';
import { BackupLogger } from '../logger/BackupLogger.logger';
import { PingService } from '../network/ping';
import { ResolveService } from '../network/resolve';
import { SchedulerConfigService } from '../scheduler/scheduler-config.service';
import { BtrfsService } from '../storage/btrfs/btrfs.service';
import { InternalBackupTask } from './tasks.class';
import { BackupTask } from './tasks.dto';
import { TasksService } from './tasks.service';

@Processor('queue')
export class HostConsumer {
  private logger = new Logger(HostConsumer.name);

  constructor(
    @InjectQueue('queue') private hostsQueue: Queue<BackupTask>,
    private hostsService: HostsService,
    private resolveService: ResolveService,
    private pingService: PingService,
    private tasksService: TasksService,
    private btrfsService: BtrfsService,
    private configService: ApplicationConfigService,
    private schedulerConfigService: SchedulerConfigService,
    private backupsService: BackupsService,
  ) {}

  @Process({
    name: 'backup',
    concurrency: 1,
  })
  async launchBackup(job: Job<BackupTask>) {
    this.logger.log(`START: Launch the backup of the host ${job.data.host} - JOB ID = ${job.id}`);

    const config = await this.updateBackupTaskConfig(job);

    await this.lock(job);

    try {
      const backupTask = job.data;
      if (backupTask.number === undefined || !backupTask.destinationDirectory) {
        const lastBackup = await this.backupsService.getLastBackup(job.data.host);
        if (lastBackup?.complete) {
          backupTask.number = lastBackup.number + 1;
          backupTask.previousDirectory = this.backupsService.getDestinationDirectory(job.data.host, lastBackup.number);
        } else {
          backupTask.number = lastBackup?.number || 0;
        }
        backupTask.destinationDirectory = this.backupsService.getDestinationDirectory(job.data.host, backupTask.number);
        job.update(backupTask);
      }

      if (!backupTask.ip) {
        backupTask.ip = await this.resolveService.resolveFromConfig(backupTask.host, config);
        if (!backupTask.ip) {
          throw new BadGatewayException(`Can't find IP for host ${backupTask.host}`);
        }
        job.update(backupTask);
      }

      // Set the logger
      const backupLogger = new BackupLogger(this.backupsService, job.data.host, job.data.number);

      const task = new InternalBackupTask(backupTask);
      this.tasksService.addSubTasks(task);
      await this.tasksService
        .launchBackup(backupLogger, task)
        .pipe(
          auditTime(5000), // FIXME: Conf
          map(async task => {
            job.update(task);
            job.progress(task.progression?.percent);
            await this.backupsService.addBackup(job.data.host, task.toBackup());
            return task;
          }),
        )
        .toPromise();

      job.update(task);
      job.progress(task.progression?.percent);

      this.backupsService.addBackup(job.data.host, task.toBackup());
    } catch (err) {
      this.logger.error(
        `END: Job for ${job.data.host} failed with error: ${err.message} - JOB ID = ${job.id}`,
        err.stack,
      );
      throw err;
    } finally {
      await this.unlock(job);
    }
    this.logger.debug(`END: Of backup of the host ${job.data.host} - JOB ID = ${job.id}`);
  }

  @Process({
    name: 'schedule_host',
    concurrency: 1,
  })
  async schedule(job: Job<BackupTask>) {
    this.logger.log(`START: Test ${job.data.host} for backup - JOB ID = ${job.id}`);

    const config = await this.updateBackupTaskConfig(job);
    const schedulerConfig = await this.schedulerConfigService.getScheduler();
    const schedule = Object.assign({}, config.schedule, schedulerConfig.defaultSchedule);

    const lockedJobId = await this.backupsService.isLocked(job.data.host);
    if (lockedJobId && (await (await this.hostsQueue.getJob(lockedJobId))?.isActive())) {
      this.logger.debug(`END: A job (${lockedJobId}) is already running for  ${job.data.host} - JOB ID = ${job.id}`);
      return;
    }

    const lastBackup = await this.backupsService.getLastBackup(job.data.host);

    // If backup is activated, and the last backup is old, we crete a new backup
    const timeSinceLastBackup = (new Date().getTime() - (lastBackup?.startDate.getTime() || 0)) / 1000;
    this.logger.debug(
      `Last backup for the host ${job.data.host} have been made at ${timeSinceLastBackup /
        3600} hours past (should be made after ${schedule.backupPerdiod / 3600} hour)  - JOB ID = ${job.id}`,
    );
    if (schedule.activated && (!lastBackup?.complete || timeSinceLastBackup > schedule.backupPerdiod)) {
      // Check if we can ping
      // Add a more complexe logic, to backup only if not in a blackout period.
      if (await this.pingService.pingFromConfig(job.data.host, config)) {
        // Yes we can, so we backup
        this.hostsQueue.add('backup', job.data, { removeOnComplete: true });
      } else {
        this.logger.debug(`END: Host ${job.data.host} not available on network - JOB ID = ${job.id}`);
      }
    } else {
      this.logger.debug(`END: Host ${job.data.host} will not be backuped - JOB ID = ${job.id}`);
    }

    // Check if some backup should be removed
  }

  @Process('remove_backup')
  async remove(job: Job<BackupTask>) {
    this.logger.debug(`START: Remove ${job.data.host} backup number ${job.data.number} - JOB ID = ${job.id}`);
    if (!job.data.host || job.data.number === undefined) {
      throw new BadRequestException(`Host and backup number should be defined`);
    }

    await this.lock(job);
    try {
      await this.backupsService.removeBackup(job.data.host, job.data.number);
      await this.btrfsService.removeSnapshot(
        await this.backupsService.getDestinationDirectory(job.data.host, job.data.number),
      );
    } finally {
      await this.unlock(job);
    }
  }

  private async lock(job: Job<BackupTask>) {
    /* *********** LOCK ************ */
    const previousLock = await this.backupsService.lock(job.data.host, job.id);
    if (previousLock) {
      const previousJob = await this.hostsQueue.getJob(previousLock);
      if (!previousJob || !(await previousJob.isActive())) {
        await this.backupsService.lock(job.data.host, job.id, true);
      } else {
        throw new Error(`Host ${job.data.host} already locked by ${previousLock}`);
      }
    }
    /* *********** END LOCK ************ */
  }

  private async unlock(job: Job<BackupTask>) {
    /* ************** UNLOCK ************ */
    await this.backupsService.unlock(job.data.host, job.id);
    /* ************** END UNLOCK ************ */
  }

  private async updateBackupTaskConfig(job: Job<BackupTask>) {
    const backupTask = job.data;

    if (!backupTask.config) {
      backupTask.config = await this.hostsService.getHostConfiguration(backupTask.host);
      if (!backupTask.config) {
        throw new NotFoundException(`Can't found ${backupTask.host}.`);
      }
      job.update(backupTask);
    }

    return backupTask.config;
  }
}
