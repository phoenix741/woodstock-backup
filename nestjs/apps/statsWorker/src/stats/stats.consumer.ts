import { Process, Processor } from '@nestjs/bull';
import { Logger } from '@nestjs/common';
import { BackupTask, StatsService } from '@woodstock/backoffice-shared';
import { Job } from 'bull';

@Processor('stats')
export class StatsConsumer {
  private logger = new Logger(StatsConsumer.name);

  constructor(private statsService: StatsService) {}

  @Process()
  async calculateStats(job: Job<BackupTask>): Promise<void> {
    this.logger.log(`START: Calculate for backup ${job.data.host}/${job.data.number} - JOB ID = ${job.id}`);

    this.logger.debug(`END: Of calculate of the host ${job.data.host}/${job.data.number} - JOB ID = ${job.id}`);
  }

  public async calculateStatsForHost(job: Job<BackupTask>, number: number): Promise<void> {
    this.logger.log(`START: Calculate for backup ${job.data.host}/${job.data.number} - JOB ID = ${job.id}`);

    this.logger.debug(`END: Of calculate of the host ${job.data.host}/${job.data.number} - JOB ID = ${job.id}`);
  }
}
