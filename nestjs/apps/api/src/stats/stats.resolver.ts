import { Query, ResolveField, Resolver } from '@nestjs/graphql';
import {
  BigIntTimeSerie,
  DiskStatisticsService,
  DiskUsage,
  HostStatistics,
  NumberTimeSerie,
  PoolStatisticsService,
  PoolUsage,
  Statistics,
  StatsInstantService,
} from '@woodstock/shared';

@Resolver(() => Statistics)
export class StatsResolver {
  constructor(
    private instantStatsService: StatsInstantService,
    private poolStatisticsService: PoolStatisticsService,
    private diskStatisticsService: DiskStatisticsService,
  ) {}

  @Query(() => Statistics)
  statistics(): Statistics {
    return {};
  }

  @ResolveField(() => DiskUsage)
  async diskUsage(): Promise<DiskUsage> {
    const { used, free, size: total } = await this.instantStatsService.getSpace();
    const statistics = await this.diskStatisticsService.readHistoryStatistics();
    const lastMonth = await this.findLastMonth(statistics);

    return {
      used,
      usedLastMonth: lastMonth?.used ?? 0n,
      usedRange: statistics.map((s) => new BigIntTimeSerie({ time: s.date, value: s.used })),
      free,
      freeLastMonth: lastMonth?.used ?? 0n,
      freeRange: statistics.map((s) => new BigIntTimeSerie({ time: s.date, value: s.free })),
      total,
      totalLastMonth: lastMonth?.used ?? 0n,
      totalRange: statistics.map((s) => new BigIntTimeSerie({ time: s.date, value: s.size })),
    };
  }

  @ResolveField(() => PoolUsage)
  async poolUsage(): Promise<PoolUsage> {
    const { longestChain, nbChunk, nbRef, size, compressedSize, unusedSize } =
      await this.instantStatsService.getPoolStatsUsage();
    const statistics = await this.poolStatisticsService.readPoolHistoryStatistics();
    const lastMonth = await this.findLastMonth(statistics);

    return {
      longestChain,
      longestChainRange: statistics.map((s) => new NumberTimeSerie({ time: s.date, value: s.longestChain })),
      longestChainLastMonth: lastMonth?.longestChain,

      nbChunk,
      nbChunkRange: statistics.map((s) => new NumberTimeSerie({ time: s.date, value: s.nbChunk })),
      nbChunkLastMonth: lastMonth?.nbChunk,

      nbRef,
      nbRefRange: statistics.map((s) => new NumberTimeSerie({ time: s.date, value: s.nbRef })),
      nbRefLastMonth: lastMonth?.nbRef,

      size,
      sizeRange: statistics.map((s) => new BigIntTimeSerie({ time: s.date, value: s.size })),
      sizeLastMonth: lastMonth?.size ?? 0n,

      compressedSize,
      compressedSizeRange: statistics.map((s) => new BigIntTimeSerie({ time: s.date, value: s.compressedSize })),
      compressedSizeLastMonth: lastMonth?.compressedSize ?? 0n,

      unusedSize: unusedSize,
      unusedSizeRange: statistics.map((s) => new BigIntTimeSerie({ time: s.date, value: s.unusedSize })),
      unusedSizeLastMonth: lastMonth?.unusedSize ?? 0n,
    };
  }

  @ResolveField(() => [HostStatistics])
  async hosts(): Promise<HostStatistics[]> {
    const hostsUsage = await this.instantStatsService.getHostsStatsUsage();

    const statistics = Object.keys(hostsUsage).map(async (host) => {
      const statistics = await this.poolStatisticsService.readHostHistoryStatistics(host);
      const lastMonth = await this.findLastMonth(statistics);

      return {
        host,

        longestChain: hostsUsage[host].longestChain,
        longestChainRange: statistics.map((s) => new NumberTimeSerie({ time: s.date, value: s.longestChain })),
        longestChainLastMonth: lastMonth?.longestChain,

        nbChunk: hostsUsage[host].nbChunk,
        nbChunkRange: statistics.map((s) => new NumberTimeSerie({ time: s.date, value: s.nbChunk })),
        nbChunkLastMonth: lastMonth?.nbChunk,

        nbRef: hostsUsage[host].nbRef,
        nbRefRange: statistics.map((s) => new NumberTimeSerie({ time: s.date, value: s.nbRef })),
        nbRefLastMonth: lastMonth?.nbRef,

        size: hostsUsage[host].size,
        sizeRange: statistics.map((s) => new BigIntTimeSerie({ time: s.date, value: s.size })),
        sizeLastMonth: lastMonth?.size ?? 0n,

        compressedSize: hostsUsage[host].compressedSize,
        compressedSizeRange: statistics.map((s) => new BigIntTimeSerie({ time: s.date, value: s.compressedSize })),
        compressedSizeLastMonth: lastMonth?.compressedSize ?? 0n,
      };
    });

    return await Promise.all(statistics);
  }

  /**
   * Find the value in the history array that is closest to the given date substract one month.
   * @param histories
   */
  private findLastMonth<T extends { date: number }>(histories: T[]): T | undefined {
    const now = new Date().getTime();
    const monthAgoTimestamp = now - (365 / 12) * 1000 * 24 * 3600;

    const history = histories.sort((h1, h2) => h1.date - h2.date).find((h) => h.date >= monthAgoTimestamp);
    if (history) {
      return history;
    }

    return undefined;
  }
}
