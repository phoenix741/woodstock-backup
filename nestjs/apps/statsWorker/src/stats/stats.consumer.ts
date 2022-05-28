import { Process, Processor } from '@nestjs/bull';
import { Logger } from '@nestjs/common';
import { BackupTask, DiskStatisticsService, StatsInstantService } from '@woodstock/shared';
import { Job } from 'bull';

@Processor('stats')
export class StatsConsumer {
  private logger = new Logger(StatsConsumer.name);

  constructor(private instantService: StatsInstantService, private statsService: DiskStatisticsService) {}

  @Process()
  async calculateStats(job: Job<BackupTask>): Promise<void> {
    this.logger.log(`START: Calculate for backup ${job.data.host}/${job.data.number} - JOB ID = ${job.id}`);

    const instant = await this.instantService.getSpace();
    await this.statsService.appendHistoryStatistics(instant);

    this.logger.debug(`END: Of calculate of the host ${job.data.host}/${job.data.number} - JOB ID = ${job.id}`);
  }
}
