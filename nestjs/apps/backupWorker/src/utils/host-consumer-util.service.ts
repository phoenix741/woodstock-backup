import { Injectable, NotFoundException } from '@nestjs/common';
import { HostConfiguration, HostsService, JobBackupData } from '@woodstock/shared';
import { Job } from 'bullmq';

@Injectable()
export class HostConsumerUtilService {
  constructor(private hostsService: HostsService) {}

  async updateBackupTaskConfig(job: Job<JobBackupData>): Promise<HostConfiguration> {
    const backupTask = job.data;

    if (!backupTask.config) {
      backupTask.config = await this.hostsService.getHost(backupTask.host);
      if (!backupTask.config) {
        throw new NotFoundException(`Can't found ${backupTask.host}.`);
      }
      job.updateData(backupTask);
    }

    return backupTask.config;
  }
}
