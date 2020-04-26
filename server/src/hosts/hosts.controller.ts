import { Body, Controller, Get, NotFoundException, Param, Post } from '@nestjs/common';
import { ApiCreatedResponse, ApiOkResponse } from '@nestjs/swagger';

import { BackupList } from '../backups/backup-list.class';
import { ApplicationConfigService } from '../config/application-config.service';
import { HostConfiguration } from './host-configuration.dto';
import { HostInformation } from './hosts.dto';
import { HostsService } from './hosts.service';

@Controller('hosts')
export class HostController {
  constructor(private configService: ApplicationConfigService, private hostsService: HostsService) {}

  @Get()
  @ApiOkResponse({
    description: 'Return the list of host',
    type: [HostInformation],
  })
  async list() {
    return Promise.all(
      (await this.hostsService.getHosts()).map(async host => {
        const list = new BackupList(this.configService.hostPath, host);
        return new HostInformation(host, await list.getLastBackup());
      }),
    );
  }

  @Get(':name')
  @ApiOkResponse({
    description: 'Return the configuration of an host',
    type: HostConfiguration,
  })
  async get(@Param('name') name: string) {
    const host = await this.hostsService.getHostConfiguration(name);
    if (!host) {
      throw new NotFoundException(`Can't find the host with name ${name}`);
    }
    return host;
  }
}
