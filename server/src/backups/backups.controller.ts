import { InjectQueue } from '@nestjs/bull';
import { Controller, Get, NotFoundException, Param, Post, Delete, ParseIntPipe } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import { ApiCreatedResponse, ApiOkResponse } from '@nestjs/swagger';
import { Queue } from 'bull';

import { HostsService } from '../hosts/hosts.service';
import { BackupTask } from '../tasks/tasks.dto';
import { BackupList } from './backup-list.class';
import { Backup } from './backup.dto';

@Controller('hosts/:name/backups')
export class BackupController {
  private hostpath: string;

  constructor(
    @InjectQueue('queue') private hostsQueue: Queue<BackupTask>,
    private hostsService: HostsService,
    configService: ConfigService,
  ) {
    this.hostpath = configService.get<string>('paths.hostPath', '<defunct>');
  }

  @Get()
  @ApiOkResponse({
    description: 'List all backup available',
    type: [Backup],
  })
  async listBackup(@Param('name') name: string) {
    if (!(await this.hostsService.getHost(name))) {
      throw new NotFoundException(`Can't find the host with the name ${name}`);
    }

    const list = new BackupList(this.hostpath, name);
    return list.getBackups();
  }

  @Post()
  @ApiCreatedResponse({ description: 'Ask to add a new backup' })
  async createBackup(@Param('name') name: string) {
    if (!(await this.hostsService.getHost(name))) {
      throw new NotFoundException(`Can't find the host with the name ${name}`);
    }

    await this.hostsQueue.add('backup', { host: name });
  }

  @Delete(':number')
  @ApiOkResponse({
    description: 'Delete a backup',
  })
  async removeBackup(@Param('name') name: string, @Param('number', ParseIntPipe) number: number) {
    if (!(await this.hostsService.getHost(name))) {
      throw new NotFoundException(`Can't find the host with the name ${name}`);
    }

    await this.hostsQueue.add('remove_backup', { host: name, number });
  }
}
