import { Processor, WorkerHost } from '@nestjs/bullmq';
import { Logger } from '@nestjs/common';
import { DiskStatisticsService, QueueName, StatsInstantService } from '@woodstock/shared';
import { Job } from 'bullmq';

@Processor(QueueName.STATS_QUEUE)
export class StatsConsumer extends WorkerHost {
  private logger = new Logger(StatsConsumer.name);

  constructor(
    private instantService: StatsInstantService,
    private statsService: DiskStatisticsService,
  ) {
    super();
  }

  async process(job: Job<void>): Promise<void> {
    this.logger.log(`START: Calculate stats for disk - JOB ID = ${job.id}`);

    const instant = await this.instantService.getSpace();
    await this.statsService.appendHistoryStatistics(instant);

    this.logger.debug(`END: Of calculate stats for disk  - JOB ID = ${job.id}`);
  }
}
