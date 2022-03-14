import { Float, Parent, ResolveField, Resolver } from '@nestjs/graphql';
import { BackupQuota } from '@woodstock/backoffice-shared';

@Resolver(() => BackupQuota)
export class BackupQuotaResolver {
  @ResolveField(() => Float)
  total(@Parent() parent: BackupQuota): number {
    return parent.excl + parent.refr;
  }
}
