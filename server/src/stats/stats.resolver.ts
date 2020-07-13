import { Query, ResolveField, Resolver } from '@nestjs/graphql';

import { CompressionStatistics, DiskUsageStats, SpaceStatistics } from './stats.model';
import { StatsService } from './stats.service';

@Resolver(() => DiskUsageStats)
export class StatsResolver {
  constructor(private statsService: StatsService) {}

  @Query(() => DiskUsageStats)
  async diskUsageStats() {
    return this.statsService.getStatistics();
  }

  @ResolveField(() => SpaceStatistics)
  async currentSpace() {
    const currentSpace = await this.statsService.getSpace({});
    return {
      timestamp: new Date().getTime(),
      ...currentSpace,
    };
  }

  @ResolveField(() => [CompressionStatistics])
  compressionStats() {
    return this.statsService.getCompressionStatistics();
  }
}
