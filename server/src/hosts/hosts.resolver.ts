import { Parent, Query, ResolveField, Resolver } from '@nestjs/graphql';

import { Backup } from '../backups/backup.dto';
import { BackupsService } from '../backups/backups.service';
import { HostConfiguration } from './host-configuration.dto';
import { Host } from './host.model';
import { HostsService } from './hosts.service';

@Resolver(() => Host)
export class HostsResolver {
  constructor(private hostsService: HostsService, private backupsService: BackupsService) {}

  @Query(() => [Host])
  async hosts(): Promise<Host[]> {
    return (await this.hostsService.getHosts()).map(host => ({
      name: host,
    }));
  }

  @ResolveField(() => HostConfiguration)
  async configuration(@Parent() host: Host): Promise<HostConfiguration> {
    return await this.hostsService.getHostConfiguration(host.name);
  }

  @ResolveField(() => [Backup])
  async backups(@Parent() host: Host): Promise<Backup[]> {
    return await this.backupsService.getBackups(host.name);
  }

  @ResolveField(() => Backup, { nullable: true })
  async lastBackup(@Parent() host: Host): Promise<Backup | undefined> {
    return await this.backupsService.getLastBackup(host.name);
  }
}
