import { Query, ResolveField, Resolver } from '@nestjs/graphql';

import { CompressionStatistics, DiskUsageStats, SpaceStatistics, HostQuota } from './stats.model';
import { StatsService } from './stats.service';
import { last } from 'rxjs/operators';

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

  @ResolveField(() => [HostQuota], { nullable: true })
  async currentRepartition() {
    const currentStatistics = await this.statsService.getStatistics();
    const lastQuotas = currentStatistics.quotas.length && currentStatistics.quotas[currentStatistics.quotas.length - 1];
    if (lastQuotas) {
      return lastQuotas.volumes.filter(v => v.number === -1).map(v => ({ ...v, total: v.refr }));
    }
    return null;
  }

  @ResolveField(() => [CompressionStatistics])
  compressionStats() {
    return this.statsService.getCompressionStatistics();
  }
}
