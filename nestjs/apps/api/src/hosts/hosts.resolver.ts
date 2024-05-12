import { InjectQueue } from '@nestjs/bullmq';
import { NotFoundException } from '@nestjs/common';
import { Args, Parent, Query, ResolveField, Resolver } from '@nestjs/graphql';
import { Backup, HostConfiguration, JobBackupData, QueueName } from '@woodstock/shared';
import { Queue } from 'bullmq';
import { Host } from './hosts.dto.js';
import { BackupsService, HostsService } from '@woodstock/shared';

@Resolver(() => Host)
export class HostsResolver {
  constructor(
    @InjectQueue(QueueName.BACKUP_QUEUE) private hostsQueue: Queue<JobBackupData>,
    private hostsService: HostsService,
    private backupsService: BackupsService,
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
}
