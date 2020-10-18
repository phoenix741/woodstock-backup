import { Float, Parent, ResolveField, Resolver } from '@nestjs/graphql';

import { BackupQuota } from './stats.model';

@Resolver(() => BackupQuota)
export class BackupQuotaResolver {
  @ResolveField(() => Float)
  total(@Parent() parent: BackupQuota): number {
    return parent.excl + parent.refr;
  }
}
