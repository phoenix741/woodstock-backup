import { Body, Controller, Get, NotFoundException, Param, Post } from '@nestjs/common';
import { ApiCreatedResponse, ApiOkResponse } from '@nestjs/swagger';

import { HostConfig } from './host-config.dto';
import { HostsService } from './hosts.service';

@Controller('hosts')
export class HostController {
  constructor(private hostsService: HostsService) {}

  @Get()
  @ApiOkResponse({
    description: 'Return the list of host',
    type: [String],
  })
  async list() {
    return (await this.hostsService.getHosts()).map(config => config.name);
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
