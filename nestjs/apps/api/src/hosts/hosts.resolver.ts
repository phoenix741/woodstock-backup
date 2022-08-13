import { NotFoundException } from '@nestjs/common';
import { Args, Parent, Query, ResolveField, Resolver } from '@nestjs/graphql';
import { Backup, BackupsService, HostConfiguration, HostsService } from '@woodstock/shared';
import { Host } from './host.model.js';

@Resolver(() => Host)
export class HostsResolver {
  constructor(private hostsService: HostsService, private backupsService: BackupsService) {}

  @Query(() => [Host])
  async hosts(): Promise<Host[]> {
    return (await this.hostsService.getHosts()).map((host) => ({
      name: host,
    }));
  }

  @Query(() => Host)
  async host(@Args('hostname') host: string): Promise<Host> {
    if (!(await this.hostsService.getHosts()).includes(host)) {
      throw new NotFoundException(`Can't find the host with the name ${host}`);
    }

    return { name: host };
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

  @ResolveField(() => String, { nullable: true })
  async lastBackupState(@Parent() host: Host): Promise<string | undefined> {
    return await this.hostsService.lastBackupState(host.name);
  }
}
