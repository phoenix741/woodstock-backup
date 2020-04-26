import { Parent, Query, ResolveField, Resolver } from '@nestjs/graphql';

import { Host } from './host.model';
import { HostsService } from './hosts.service';
import { HostConfiguration } from './host-configuration.dto';

@Resolver(() => Host)
export class HostsResolver {
  constructor(private hostsService: HostsService) {}

  @Query(() => [Host])
  async hosts(): Promise<Host[]> {
    return (await this.hostsService.getHosts()).map(host => ({
      name: host,
    }));
  }

  @ResolveField(() => HostConfiguration)
  async configuration(@Parent() host: Host): Promise<HostConfiguration> {
    return (await this.hostsService.getHostConfiguration(host.name)) || {};
  }
}
