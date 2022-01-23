import { InjectQueue } from '@nestjs/bull';
import { Injectable, NotFoundException } from '@nestjs/common';
import { BackupsService, BackupTask, HostConfiguration, HostsService } from '@woodstock/backoffice-shared';
import { Job, Queue } from 'bull';

@Injectable()
export class HostConsumerUtilService {
  constructor(
    @InjectQueue('queue') private hostsQueue: Queue<BackupTask>,
    private hostsService: HostsService,
    private backupsService: BackupsService,
  ) {}

  async lock(job: Job<BackupTask>): Promise<void> {
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

  async unlock(job: Job<BackupTask>): Promise<void> {
    /* ************** UNLOCK ************ */
    await this.backupsService.unlock(job.data.host, job.id);
    /* ************** END UNLOCK ************ */
  }

  async updateBackupTaskConfig(job: Job<BackupTask>): Promise<HostConfiguration> {
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
