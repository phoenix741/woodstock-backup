import { Processor, WorkerHost } from '@nestjs/bullmq';
import { Logger } from '@nestjs/common';
import { DiskStatisticsService, StatsInstantService } from '@woodstock/shared';
import { JobBackupData } from '@woodstock/shared/backuping/backuping.model';
import { Job } from 'bullmq';

@Processor('stats')
export class StatsConsumer extends WorkerHost {
  private logger = new Logger(StatsConsumer.name);

  constructor(private instantService: StatsInstantService, private statsService: DiskStatisticsService) {
    super();
  }

  async process(job: Job<JobBackupData>): Promise<void> {
    this.logger.log(`START: Calculate for backup ${job.data.host}/${job.data.number} - JOB ID = ${job.id}`);

    const instant = await this.instantService.getSpace();
    await this.statsService.appendHistoryStatistics(instant);

    this.logger.debug(`END: Of calculate of the host ${job.data.host}/${job.data.number} - JOB ID = ${job.id}`);
  }
}
