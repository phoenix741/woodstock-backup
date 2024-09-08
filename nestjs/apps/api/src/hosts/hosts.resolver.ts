import { InjectQueue } from '@nestjs/bullmq';
import { NotFoundException, Res } from '@nestjs/common';
import { Args, Float, Int, Parent, Query, ResolveField, Resolver } from '@nestjs/graphql';
import { Backup, HostConfiguration, JobBackupData, JobService, QueueName, ResolveService } from '@woodstock/shared';
import { Queue } from 'bullmq';
import { Host } from './hosts.dto.js';
import { BackupsService, HostsService } from '@woodstock/shared';

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
  async backups(@Parent() host: Host): Promise<Backup[]> {
    return await this.backupsService.getBackups(host.name);
  }

  @ResolveField(() => Backup, { nullable: true })
  async lastBackup(@Parent() host: Host): Promise<Backup | undefined> {
    return (await this.backupsService.getLastBackup(host.name)) ?? undefined;
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

  @ResolveField(() => [String], { nullable: true })
  async addresses(@Parent() host: Host): Promise<string[] | undefined> {
    const configuration = await this.hostsService.getHost(host.name);
    const addresses = await this.resolveService.resolveFromConfig(host.name, configuration);
    return addresses;
  }
}
