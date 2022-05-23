import { Query, ResolveField, Resolver } from '@nestjs/graphql';
import {
  DiskStatisticsService,
  DiskUsage,
  HostStatistics,
  PoolStatisticsService,
  PoolUsage,
  Statistics,
  StatsInstantService,
  TimeSerie,
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
    const { used, free, size } = await this.instantStatsService.getSpace();
    const statistics = await this.diskStatisticsService.readHistoryStatistics();
    const lastMonth = await this.findLastMonth(statistics);

    return {
      used: Number(used / 1024n / 1024n),
      usedLastMonth: Number((lastMonth?.used || 0n) / 1024n / 1024n),
      usedRange: statistics.map((s) => new TimeSerie({ time: s.date, value: Number(s.used / 1024n / 1024n) })),
      free: Number(free / 1024n / 1024n),
      freeLastMonth: Number((lastMonth?.used || 0n) / 1024n / 1024n),
      freeRange: statistics.map((s) => new TimeSerie({ time: s.date, value: Number(s.free / 1024n / 1024n) })),
      total: Number(size / 1024n / 1024n),
      totalLastMonth: Number((lastMonth?.used || 0n) / 1024n / 1024n),
      totalRange: statistics.map((s) => new TimeSerie({ time: s.date, value: Number(s.size / 1024n / 1024n) })),
    };
  }

  @ResolveField(() => PoolUsage)
  async poolUsage(): Promise<PoolUsage> {
    const { longestChain, nbChunk, nbRef, size, compressedSize } = await this.instantStatsService.getPoolStatsUsage();
    const statistics = await this.poolStatisticsService.readPoolHistoryStatistics();
    const lastMonth = await this.findLastMonth(statistics);

    return {
      longestChain,
      longestChainRange: statistics.map((s) => new TimeSerie({ time: s.date, value: s.longestChain })),
      longestChainLastMonth: lastMonth?.longestChain,

      nbChunk,
      nbChunkRange: statistics.map((s) => new TimeSerie({ time: s.date, value: s.nbChunk })),
      nbChunkLastMonth: lastMonth?.nbChunk,

      nbRef,
      nbRefRange: statistics.map((s) => new TimeSerie({ time: s.date, value: s.nbRef })),
      nbRefLastMonth: lastMonth?.nbRef,

      size: Number(size / 1024n / 1024n),
      sizeRange: statistics.map((s) => new TimeSerie({ time: s.date, value: Number(s.size / 1024n / 1024n) })),
      sizeLastMonth: Number((lastMonth?.size || 0n) / 1024n / 1024n),

      compressedSize: Number(compressedSize / 1024n / 1024n),
      compressedSizeRange: statistics.map(
        (s) => new TimeSerie({ time: s.date, value: Number(s.compressedSize / 1024n / 1024n) }),
      ),
      compressedSizeLastMonth: Number((lastMonth?.compressedSize || 0n) / 1024n / 1024n),
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
        longestChainRange: statistics.map((s) => new TimeSerie({ time: s.date, value: s.longestChain })),
        longestChainLastMonth: lastMonth?.longestChain,

        nbChunk: hostsUsage[host].nbChunk,
        nbChunkRange: statistics.map((s) => new TimeSerie({ time: s.date, value: s.nbChunk })),
        nbChunkLastMonth: lastMonth?.nbChunk,

        nbRef: hostsUsage[host].nbRef,
        nbRefRange: statistics.map((s) => new TimeSerie({ time: s.date, value: s.nbRef })),
        nbRefLastMonth: lastMonth?.nbRef,

        size: Number(hostsUsage[host].size / 1024n / 1024n),
        sizeRange: statistics.map((s) => new TimeSerie({ time: s.date, value: Number(s.size / 1024n / 1024n) })),
        sizeLastMonth: Number((lastMonth?.size || 0n) / 1024n / 1024n),

        compressedSize: Number(hostsUsage[host].compressedSize / 1024n / 1024n),
        compressedSizeRange: statistics.map(
          (s) => new TimeSerie({ time: s.date, value: Number(s.compressedSize / 1024n / 1024n) }),
        ),
        compressedSizeLastMonth: Number((lastMonth?.compressedSize || 0n) / 1024n / 1024n),
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
