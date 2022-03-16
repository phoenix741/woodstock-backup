import { InjectQueue } from '@nestjs/bull';
import { ClassSerializerInterceptor, NotFoundException, UseInterceptors } from '@nestjs/common';
import { Args, Int, Mutation, Parent, Query, ResolveField, Resolver } from '@nestjs/graphql';
import { Backup, BackupsService, BackupTask, FileDescription, HostsService } from '@woodstock/backoffice-shared';
import { Queue } from 'bull';
import { BackupsFilesService } from './backups-files.service';
import { JobResponse } from './backups.model';

interface ExtendedBackup extends Backup {
  hostname: string;
}

@Resolver(() => Backup)
export class BackupsResolver {
  constructor(
    @InjectQueue('queue') private hostsQueue: Queue<BackupTask>,
    private service: BackupsFilesService,
    private hostsService: HostsService,
    private backupsService: BackupsService,
  ) {}

  @Query(() => [Backup])
  async backups(@Args('hostname') hostname: string): Promise<ExtendedBackup[]> {
    if (!(await this.hostsService.getHosts()).includes(hostname)) {
      throw new NotFoundException(`Can't find the host with the name ${hostname}`);
    }

    const backups = await this.backupsService.getBackups(hostname);

    return backups.map((backup) => Object.assign({ hostname }, backup));
  }

  @Query(() => Backup)
  async backup(
    @Args('hostname') hostname: string,
    @Args('number', { type: () => Int }) number: number,
  ): Promise<ExtendedBackup> {
    if (!(await this.hostsService.getHosts()).includes(hostname)) {
      throw new NotFoundException(`Can't find the host with the name ${hostname}`);
    }

    const backups = await this.backupsService.getBackups(hostname);
    const backup = backups.find((backup) => backup.number === number);
    if (!backup) {
      throw new NotFoundException(`Can't find the backup ${number} for the host ${hostname}`);
    }

    return Object.assign({ hostname }, backup);
  }

  @ResolveField(() => [FileDescription])
  @UseInterceptors(ClassSerializerInterceptor)
  async shares(@Parent() parent: ExtendedBackup): Promise<FileDescription[]> {
    return this.service.listShare(parent.hostname, parent.number);
  }

  @ResolveField(() => [FileDescription])
  @UseInterceptors(ClassSerializerInterceptor)
  async files(
    @Parent() parent: ExtendedBackup,
    @Args('sharePath') sharePath: string,
    @Args('path') path: string,
  ): Promise<FileDescription[]> {
    return this.service.list(parent.hostname, parent.number, sharePath, path);
  }

  @Mutation(() => JobResponse)
  async createBackup(@Args('hostname') hostname: string): Promise<JobResponse> {
    if (!(await this.hostsService.getHosts()).includes(hostname)) {
      throw new NotFoundException(`Can't find the host with the name ${hostname}`);
    }

    const job = await this.hostsQueue.add('backup', { host: hostname }, { removeOnComplete: false });
    return {
      id: job.id as number,
    };
  }

  @Mutation(() => JobResponse)
  async removeBackup(
    @Args('hostname') hostname: string,
    @Args('number', { type: () => Int }) number: number,
  ): Promise<JobResponse> {
    if (!(await this.hostsService.getHosts()).includes(hostname)) {
      throw new NotFoundException(`Can't find the host with the name ${hostname}`);
    }

    const job = await this.hostsQueue.add('remove_backup', { host: hostname, number }, { removeOnComplete: true });
    return {
      id: job.id as number,
    };
  }
}
