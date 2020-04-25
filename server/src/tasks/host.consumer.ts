import { InjectQueue, Process, Processor } from '@nestjs/bull';
import { BadGatewayException, Logger, NotFoundException, BadRequestException } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import { Job, Queue } from 'bull';

import { BackupList } from '../backups/backup-list.class';
import { HostsService } from '../hosts/hosts.service';
import { PingService } from '../network/ping';
import { ResolveService } from '../network/resolve';
import { InternalBackupTask } from './tasks.class';
import { BackupTask } from './tasks.dto';
import { TasksService } from './tasks.service';
import { BtrfsService } from '../storage/btrfs/btrfs.service';
import { BackupLogger } from '../logger/BackupLogger.logger';

@Processor('queue')
export class HostConsumer {
  private logger = new Logger(HostConsumer.name);
  private hostpath: string;

  constructor(
    @InjectQueue('queue') private hostsQueue: Queue<BackupTask>,
    private hostsService: HostsService,
    private resolveService: ResolveService,
    private pingService: PingService,
    private tasksService: TasksService,
    private btrfsService: BtrfsService,
    private configService: ConfigService,
  ) {
    this.hostpath = configService.get<string>('paths.hostPath', '<defunct>');
  }

  @Process('backup')
  async launchBackup(job: Job<BackupTask>) {
    this.logger.log(`START: Launch the backup of the host ${job.data.host} - JOB ID = ${job.id}`);

    const config = await this.updateBackupTaskConfig(job);

    await this.lock(job);

    try {
      const backupTask = job.data;
      const hostBackup = new BackupList(this.hostpath, job.data.host);
      if (backupTask.number === undefined || !backupTask.destinationDirectory) {
        const lastBackup = await hostBackup.getLastBackup();
        if (lastBackup?.complete) {
          backupTask.number = lastBackup.number + 1;
          backupTask.previousDirectory = hostBackup.getDestinationDirectory(lastBackup.number);
        } else {
          backupTask.number = lastBackup?.number || 0;
        }
        backupTask.destinationDirectory = hostBackup.getDestinationDirectory(backupTask.number);
        job.update(backupTask);
      }

      if (!backupTask.ip) {
        backupTask.ip = await this.resolveService.resolveFromConfig(config);
        if (!backupTask.ip) {
          throw new BadGatewayException(`Can't find IP for host ${backupTask.host}`);
        }
        job.update(backupTask);
      }

      // Set the logger
      const backupLogger = new BackupLogger(this.configService, job.data.host, job.data.number);

      const task = new InternalBackupTask(backupTask);
      this.tasksService.addSubTasks(task);
      await this.tasksService.launchBackup(backupLogger, task, task => {
        job.update(task);
        job.progress(task.progression?.percent);
        hostBackup.addBackup(task.toBackup());
      });

      hostBackup.addBackup(task.toBackup());
    } finally {
      await this.unlock(job);
    }
    this.logger.debug(`END: Of backup of the host ${job.data.host} - JOB ID = ${job.id}`);
  }

  @Process('schedule_host')
  async schedule(job: Job<BackupTask>) {
    this.logger.log(`START: Test ${job.data.host} for backup - JOB ID = ${job.id}`);

    const config = await this.updateBackupTaskConfig(job);

    const hostBackup = new BackupList(this.hostpath, job.data.host);
    const lastBackup = await hostBackup.getLastBackup();

    // If backup is activated, and the last backup is old, we crete a new backup
    const timeSinceLastBackup = (new Date().getTime() - (lastBackup?.startDate.getTime() || 0)) / 1000;
    this.logger.debug(
      `Last backup for the host ${job.data.host} have been made at ${timeSinceLastBackup /
        3600} hours past (should be made after ${config.schedule.backupPerdiod / 3600} hour)  - JOB ID = ${job.id}`,
    );
    if (config.schedule.activated && timeSinceLastBackup > config.schedule.backupPerdiod) {
      // Check if we can ping
      // Add a more complexe logic, to backup only if not in a blackout period.
      if (await this.pingService.pingFromConfig(config)) {
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
      const hostBackup = new BackupList(this.hostpath, job.data.host);
      await hostBackup.removeBackup(job.data.number);
      await this.btrfsService.removeSnapshot(await hostBackup.getDestinationDirectory(job.data.number));
    } finally {
      await this.unlock(job);
    }
  }

  private async lock(job: Job<BackupTask>) {
    /* *********** LOCK ************ */
    const hostBackup = new BackupList(this.hostpath, job.data.host);
    const previousLock = await hostBackup.lock('' + job.id);
    if (previousLock) {
      const previousJob = await this.hostsQueue.getJob(previousLock);
      if (!previousJob || !(await previousJob.isActive())) {
        await hostBackup.lock('' + job.id, true);
      } else {
        throw new Error(`Host ${job.data.host} already locked by ${previousLock}`);
      }
    }
    /* *********** END LOCK ************ */
  }

  private async unlock(job: Job<BackupTask>) {
    const hostBackup = new BackupList(this.hostpath, job.data.host);
    /* ************** UNLOCK ************ */
    await hostBackup.unlock('' + job.id);
    /* ************** END UNLOCK ************ */
  }

  private async updateBackupTaskConfig(job: Job<BackupTask>) {
    const backupTask = job.data;

    if (!backupTask.config) {
      backupTask.config = await this.hostsService.getHost(backupTask.host);
      if (!backupTask.config) {
        throw new NotFoundException(`Can't found ${backupTask.host}.`);
      }
      job.update(backupTask);
    }

    return backupTask.config;
  }
}
