import { Query, ResolveField, Resolver } from '@nestjs/graphql';
import {
  CompressionStatistics,
  DiskUsageStats,
  HostQuota,
  SpaceStatistics,
  StatsService,
} from '@woodstock/backoffice-shared';

@Resolver(() => DiskUsageStats)
export class StatsResolver {
  constructor(private statsService: StatsService) {}

  @Query(() => DiskUsageStats)
  async diskUsageStats(): Promise<DiskUsageStats> {
    return this.statsService.getStatistics();
  }

  @ResolveField(() => SpaceStatistics)
  async currentSpace(): Promise<{ timestamp: number; fstype: string; size: number; used: number; free: number }> {
    const currentSpace = await this.statsService.getSpace({});
    return {
      timestamp: new Date().getTime(),
      ...currentSpace,
    };
  }

  @ResolveField(() => [HostQuota], { nullable: true })
  async currentRepartition(): Promise<
    | {
        total: number;
        host: string;
        number: number;
        refr: number;
        excl: number;
      }[]
    | null
  > {
    const currentStatistics = await this.statsService.getStatistics();
    const lastQuotas = currentStatistics.quotas.length && currentStatistics.quotas[currentStatistics.quotas.length - 1];
    if (lastQuotas) {
      return lastQuotas.volumes.filter((v) => v.number === -1).map((v) => ({ ...v, total: v.refr }));
    }
    return null;
  }

  @ResolveField(() => [CompressionStatistics])
  compressionStats(): Promise<CompressionStatistics[]> {
    return this.statsService.getCompressionStatistics();
  }
}
