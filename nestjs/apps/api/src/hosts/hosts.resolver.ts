import { InjectQueue } from '@nestjs/bullmq';
import { NotFoundException } from '@nestjs/common';
import { Args, Float, Parent, Query, ResolveField, Resolver } from '@nestjs/graphql';
import {
  Backup,
  BackupsService,
  HostConfiguration,
  HostsService,
  JobBackupData,
  JobService,
  QueueName,
  ResolveService,
} from '@woodstock/shared';
import { Queue } from 'bullmq';
import { Host, HostAvailibilityState } from './hosts.dto.js';
import { ExtendedBackup } from '../backups/backups.resolver.js';

@Resolver(() => Host)
export class HostsResolver {
  constructor(
    @InjectQueue(QueueName.BACKUP_QUEUE) private hostsQueue: Queue<JobBackupData>,
    private hostsService: HostsService,
    private backupsService: BackupsService,
    private jobService: JobService,
    private resolveService: ResolveService,
  ) {}

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
    return await this.hostsService.getHost(host.name);
  }

  @ResolveField(() => [Backup])
  async backups(@Parent() host: Host): Promise<ExtendedBackup[]> {
    return (await this.backupsService.getBackups(host.name)).map((backup) => ({
      hostname: host.name,
      ...backup,
    }));
  }

  @ResolveField(() => Backup, { nullable: true })
  async lastBackup(@Parent() host: Host): Promise<ExtendedBackup | undefined> {
    const lastBackup = await this.backupsService.getLastBackup(host.name);

    if (lastBackup) {
      return { hostname: host.name, ...lastBackup };
    }
    return undefined;
  }

  @ResolveField(() => Float, { nullable: true })
  async timeSinceLastBackup(@Parent() host: Host): Promise<number | undefined> {
    return await this.backupsService.getTimeSinceLastBackup(host.name);
  }

  @ResolveField(() => Float, { nullable: true })
  async timeToNextBackup(@Parent() host: Host): Promise<number | undefined> {
    if (!(await this.jobService.isBackupRunning(host.name))) {
      return await this.jobService.getTimeToNextBackup(host.name);
    }
    return undefined;
  }

  @ResolveField(() => Date, { nullable: true })
  async dateToNextBackup(@Parent() host: Host): Promise<Date | undefined> {
    if (!(await this.jobService.isBackupRunning(host.name))) {
      return await this.jobService.getDateToNextBackup(host.name);
    }
    return undefined;
  }

  @ResolveField(() => String, { nullable: true })
  async lastBackupState(@Parent() host: Host): Promise<string | undefined> {
    const jobs = await this.hostsQueue.getJobs([]);
    const backupJobs = jobs
      .filter((j) => j.name === QueueName.BACKUP_QUEUE && j.data.host === host.name)
      .sort((j1, j2) => j2.timestamp - j1.timestamp);

    if (backupJobs[0]) {
      return await backupJobs[0].getState();
    }
    return undefined;
  }

  @ResolveField(() => String, { nullable: true })
  async agentVersion(@Parent() host: Host): Promise<string | undefined> {
    const informations = await this.resolveService.getInformations(host.name);
    return informations?.version;
  }

  @ResolveField(() => HostAvailibilityState, { nullable: true })
  async availibilityState(@Parent() host: Host): Promise<HostAvailibilityState | undefined> {
    const informations = await this.resolveService.getInformations(host.name);
    if (!informations) {
      const configuration = await this.hostsService.getHost(host.name);

      if (configuration?.addresses) {
        // If the host has addresses, we can't known if it's online or not
        return HostAvailibilityState.Unknown;
      }

      return HostAvailibilityState.Offline;
    }

    return informations.isOnline ? HostAvailibilityState.Online : HostAvailibilityState.Offline;
  }

  @ResolveField(() => [String], { nullable: true })
  async addresses(@Parent() host: Host): Promise<string[] | undefined> {
    const configuration = await this.hostsService.getHost(host.name);
    const addresses = await this.resolveService.resolveFromConfig(host.name, configuration);
    return addresses;
  }
}
