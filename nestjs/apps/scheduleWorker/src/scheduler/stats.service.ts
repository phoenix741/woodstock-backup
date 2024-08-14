import { Injectable, Logger } from '@nestjs/common';
import { DiskStatisticsService, StatsInstantService } from '@woodstock/shared';

@Injectable()
export class StatsService {
  private logger = new Logger(StatsService.name);

  constructor(
    private instantService: StatsInstantService,
    private statsService: DiskStatisticsService,
  ) {}

  async calculateSpaceStats(): Promise<void> {
    this.logger.log(`START: Calculate stats for disk`);

    const instant = await this.instantService.getSpace();
    await this.statsService.appendHistoryStatistics(instant);

    this.logger.debug(`END: Of calculate stats for disk`);
  }
}
