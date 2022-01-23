import { Controller, Get, NotFoundException, Param } from '@nestjs/common';
import { ApiOkResponse } from '@nestjs/swagger';
import { BackupsService, HostConfiguration, HostsService } from '@woodstock/backoffice-shared';
import { HostInformation } from './hosts.dto';

@Controller('hosts')
export class HostController {
  constructor(private backupsService: BackupsService, private hostsService: HostsService) {}

  @Get()
  @ApiOkResponse({
    description: 'Return the list of host',
    type: [HostInformation],
  })
  async list(): Promise<HostInformation[]> {
    return Promise.all(
      (await this.hostsService.getHosts()).map(async (host) => {
        return new HostInformation(host, await this.backupsService.getLastBackup(host));
      }),
    );
  }

  @Get(':name')
  @ApiOkResponse({
    description: 'Return the configuration of an host',
    type: HostConfiguration,
  })
  async get(@Param('name') name: string): Promise<HostConfiguration> {
    const host = await this.hostsService.getHostConfiguration(name);
    if (!host) {
      throw new NotFoundException(`Can't find the host with name ${name}`);
    }
    return host;
  }
}
