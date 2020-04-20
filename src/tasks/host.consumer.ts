import { InjectQueue, Process, Processor } from '@nestjs/bull';
import { BadGatewayException, Logger, NotFoundException } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import { Job, Queue } from 'bull';

import { BackupList } from '../backups/backup-list.class';
import { HostsService } from '../hosts/hosts.service';
import { PingService } from '../network/ping';
import { ResolveService } from '../network/resolve';
import { InternalBackupTask } from './tasks.class';
import { BackupTask } from './tasks.dto';
import { TasksService } from './tasks.service';

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
    configService: ConfigService,
  ) {
    this.hostpath = configService.get<string>('paths.hostPath', '<defunct>');
  }

  @Process('backup')
  async launchBackup(job: Job<BackupTask>) {
    this.logger.log(`START: Launch the backup of the host ${job.data.host}`);

    const config = await this.updateBackupTaskConfig(job);

    const backupTask = job.data;
    const hostBackup = new BackupList(this.hostpath, job.data.host);

    /* *********** LOCK ************ */
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

    try {
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

      const task = new InternalBackupTask(backupTask);
      this.tasksService.addSubTasks(task);
      await this.tasksService.launchBackup(task, task => {
        job.update(task);
        job.progress(task.progression?.percent);
      });

      hostBackup.addBackup(task.toBackup());
    } finally {
      /* ************** UNLOCK ************ */
      await hostBackup.unlock('' + job.id);
      /* ************** END UNLOCK ************ */
    }
    this.logger.log(`END: Of backup of the host ${job.data.host}`);
  }

  @Process('schedule_host')
  async schedule(job: Job<BackupTask>) {
    this.logger.log(`START: Test ${job.data.host} for backup`);

    const config = await this.updateBackupTaskConfig(job);

    const hostBackup = new BackupList(this.hostpath, job.data.host);
    const lastBackup = await hostBackup.getLastBackup();

    // If backup is activated, and the last backup is old, we crete a new backup
    const timeSinceLastBackup = (new Date().getTime() - (lastBackup?.startDate.getTime() || 0)) / 1000;
    this.logger.log(`Last backup for the host ${job.data.host} have been made at ${timeSinceLastBackup / 3600} hours past (should be made after ${config.schedule.backupPerdiod / 3600} hour)`);
    if (config.schedule.activated && timeSinceLastBackup > config.schedule.backupPerdiod) {
      // Check if we can ping
      // Add a more complexe logic, to backup only if not in a blackout period.
      if (await this.pingService.pingFromConfig(config)) {
        // Yes we can, so we backup
        this.hostsQueue.add('backup', job.data);
      } else {
        this.logger.warn(`END: Host ${job.data.host} not available on network`);
      }
    } else {
      this.logger.log(`END: Host ${job.data.host} will not be backuped`);
    }

    // Check if some backup should be removed
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
