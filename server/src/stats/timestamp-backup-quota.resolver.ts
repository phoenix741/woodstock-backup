import { Args, Parent, ResolveField, Resolver } from '@nestjs/graphql';

import { BackupQuota, HostQuota, TimestampBackupQuota, TotalQuota, SpaceStatistics } from './stats.model';
import { StatsService } from './stats.service';

@Resolver(() => TimestampBackupQuota)
export class TimestampBackupQuotaResolver {
  constructor(private statsService: StatsService) {}

  @ResolveField(() => [BackupQuota])
  volumes(@Parent() parent: TimestampBackupQuota, @Args('host', { type: () => String, nullable: true }) host?: string) {
    return (parent.volumes || []).filter(v => v.host === host || !host);
  }

  @ResolveField(() => [HostQuota])
  host(@Parent() parent: TimestampBackupQuota, @Args('host', { type: () => String, nullable: true }) host?: string) {
    return (parent.volumes || [])
      .filter(v => v.host === host || !host)
      .reduce((acc, v) => {
        let find = acc.find(a => a.host === v.host);
        if (!find) {
          find = { host: v.host, excl: v.excl, refr: v.refr, total: v.excl + v.refr };
          acc.push(find);
        } else {
          find.excl += v.excl;
          find.refr += v.refr;
          find.total += v.excl + v.refr;
        }

        return acc;
      }, [] as HostQuota[]);
  }

  @ResolveField(() => TotalQuota)
  async total(@Parent() parent: TimestampBackupQuota) {
    const stats = await this.statsService.getStatistics();
    let spaceStatistics: Partial<SpaceStatistics> | undefined = stats.spaces.find(
      s => s.timestamp === parent.timestamp,
    );
    if (!spaceStatistics) {
      spaceStatistics = await await this.statsService.getSpace({});
    }
    const used = spaceStatistics.used;

    return (parent.volumes || []).reduce(
      (acc, v) => {
        acc.excl += v.excl;
        acc.refr -= v.excl;
        acc.total += v.excl + v.refr;

        return acc;
      },
      { total: 0, refr: used, excl: 0 } as TotalQuota,
    );
  }
}
