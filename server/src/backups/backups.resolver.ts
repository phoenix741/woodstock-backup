import { NotFoundException } from '@nestjs/common';
import { Args, Parent, Query, ResolveField, Resolver, Mutation } from '@nestjs/graphql';

import { Backup } from '../backups/backup.dto';
import { BackupsService } from '../backups/backups.service';
import { FileDescription } from './backups-files.dto';
import { BackupsFilesService } from './backups-files.service';
import { HostsService } from '../hosts/hosts.service';
import { InjectQueue } from '@nestjs/bull';
import { Queue } from 'bull';
import { BackupTask } from '../tasks/tasks.dto';
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

  @Query(() => Backup)
  async backup(@Args('hostname') hostname: string, @Args('number') number: number): Promise<ExtendedBackup> {
    if (!(await this.hostsService.getHosts()).includes(hostname)) {
      throw new NotFoundException(`Can't find the host with the name ${hostname}`);
    }

    const backups = await this.backupsService.getBackups(hostname);
    const backup = backups.find(backup => backup.number === number);
    if (!backup) {
      throw new NotFoundException(`Can't find the backup ${number} for the host ${hostname}`);
    }

    return Object.assign({ hostname }, backup);
  }

  @ResolveField(() => [FileDescription])
  async files(@Parent() parent: ExtendedBackup, @Args('path') path: string): Promise<FileDescription[]> {
    return this.service.list(parent.hostname, parent.number, path);
  }

  @Mutation(() => JobResponse)
  async createBackup(@Args('hostname') hostname: string): Promise<JobResponse> {
    if (!(await this.hostsService.getHosts()).includes(hostname)) {
      throw new NotFoundException(`Can't find the host with the name ${hostname}`);
    }

    const job = await this.hostsQueue.add('backup', { host: hostname }, { removeOnComplete: true });
    return {
      id: job.id as number,
    };
  }

  @Mutation(() => JobResponse)
  async removeBackup(@Args('hostname') hostname: string, @Args('number') number: number): Promise<JobResponse> {
    if (!(await this.hostsService.getHosts()).includes(hostname)) {
      throw new NotFoundException(`Can't find the host with the name ${hostname}`);
    }

    const job = await this.hostsQueue.add('remove_backup', { host: hostname, number }, { removeOnComplete: true });
    return {
      id: job.id as number,
    };
  }
}
