import { Args, Parent, ResolveField, Resolver } from '@nestjs/graphql';

import { BackupQuota, HostQuota, TimestampBackupQuota, TotalQuota } from './stats.model';
import { StatsService } from './stats.service';

@Resolver(() => TimestampBackupQuota)
export class TimestampBackupQuotaResolver {
  constructor(private statsService: StatsService) {}

  @ResolveField(() => [BackupQuota])
  volumes(
    @Parent() parent: TimestampBackupQuota,
    @Args('host', { type: () => String, nullable: true }) host?: string,
  ): BackupQuota[] {
    return (parent.volumes || []).filter((v) => (v.host === host || !host) && v.number !== -1);
  }

  @ResolveField(() => [HostQuota])
  host(
    @Parent() parent: TimestampBackupQuota,
    @Args('host', { type: () => String, nullable: true }) host?: string,
  ): { host: string; number: number; excl: number; refr: number; total: number }[] {
    return (parent.volumes || [])
      .filter((v) => (v.host === host || !host) && v.number === -1)
      .map((v) => ({ ...v, total: v.refr }));
  }

  @ResolveField(() => TotalQuota)
  async total(@Parent() parent: TimestampBackupQuota): Promise<TotalQuota> {
    const { used } = parent.space;

    return (parent.volumes || [])
      .filter((v) => v.number !== -1)
      .reduce(
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
