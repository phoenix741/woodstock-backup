import { InjectQueue } from '@nestjs/bullmq';
import { Injectable, NotFoundException } from '@nestjs/common';
import { BackupsService, BackupTask, HostConfiguration, HostsService, LockService } from '@woodstock/shared';
import { Job, Queue } from 'bullmq';

@Injectable()
export class HostConsumerUtilService {
  constructor(
    @InjectQueue('queue') private hostsQueue: Queue<BackupTask>,
    private lockService: LockService,
    private hostsService: HostsService,
    private backupsService: BackupsService,
  ) {}

  async lock(job: Job<BackupTask>): Promise<void> {
    if (!job.id) {
      throw new NotFoundException('Job ID not found');
    }
    /* *********** LOCK ************ */
    const lockFile = this.backupsService.getLockFile(job.data.host);
    const previousLock = await this.lockService.lock(lockFile, job.id);
    if (previousLock) {
      const previousJob = await this.hostsQueue.getJob(previousLock);
      if (!previousJob || !(await previousJob.isActive())) {
        await this.lockService.lock(lockFile, job.id, true);
      } else {
        throw new Error(`Host ${job.data.host} already locked by ${previousLock}`);
      }
    }
    /* *********** END LOCK ************ */
  }

  async unlock(job: Job<BackupTask>): Promise<void> {
    if (!job.id) {
      throw new NotFoundException('Job ID not found');
    }
    /* ************** UNLOCK ************ */
    const lockFile = this.backupsService.getLockFile(job.data.host);
    await this.lockService.unlock(lockFile, job.id);
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
