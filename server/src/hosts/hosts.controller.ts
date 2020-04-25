import { Body, Controller, Get, NotFoundException, Param, Post } from '@nestjs/common';
import { ApiCreatedResponse, ApiOkResponse } from '@nestjs/swagger';

import { HostConfig } from './host-config.dto';
import { HostsService } from './hosts.service';
import { HostInformation } from './hosts.dto';
import { BackupList } from '../backups/backup-list.class';
import { ConfigService } from '@nestjs/config';

@Controller('hosts')
export class HostController {
  hostpath: string;

  constructor(private configService: ConfigService, private hostsService: HostsService) {
    this.hostpath = configService.get<string>('paths.hostPath', '<defunct>');
  }

  @Get()
  @ApiOkResponse({
    description: 'Return the list of host',
    type: [HostInformation],
  })
  async list() {
    return Promise.all(
      (await this.hostsService.getHosts()).map(async config => {
        const list = new BackupList(this.hostpath, config.name);
        return new HostInformation(config.name, await list.getLastBackup());
      }),
    );
  }

  @Get(':name')
  @ApiOkResponse({
    description: 'Return the configuration of an host',
    type: HostConfig,
  })
  async get(@Param('name') name: string) {
    const host = await this.hostsService.getHost(name);
    if (!host) {
      throw new NotFoundException(`Can't find the host with name ${name}`);
    }
    return host;
  }

  @Post()
  @ApiCreatedResponse({
    description: 'Add an host to the configuration',
  })
  async create(@Body() hostConfig: HostConfig) {
    await this.hostsService.addHost(hostConfig);
  }
}
