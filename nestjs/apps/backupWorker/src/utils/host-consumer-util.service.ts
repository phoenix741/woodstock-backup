import { Injectable, NotFoundException } from '@nestjs/common';
import { HostConfiguration, HostsService } from '@woodstock/shared';
import { JobBackupData } from '@woodstock/shared/backuping/backuping.model';
import { Job } from 'bullmq';

@Injectable()
export class HostConsumerUtilService {
  constructor(private hostsService: HostsService) {}

  async updateBackupTaskConfig(job: Job<JobBackupData>): Promise<HostConfiguration> {
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
