import {
  ClassSerializerInterceptor,
  Controller,
  Get,
  Headers,
  NotFoundException,
  Param,
  Res,
  UnsupportedMediaTypeException,
  UseInterceptors,
} from '@nestjs/common';
import { ApiHeader, ApiOkResponse, ApiProduces } from '@nestjs/swagger';
import { ApplicationConfigService, YamlService } from '@woodstock/shared';
import { HostConfiguration } from '@woodstock/shared';
import { CertificateService } from '@woodstock/shared';
import * as archiver from 'archiver';
import { Response } from 'express';
import { join } from 'path';
import { HostInformation } from './hosts.dto.js';
import { BackupsService, HostsService } from '@woodstock/shared';

@UseInterceptors(ClassSerializerInterceptor)
@Controller('hosts')
export class HostController {
  constructor(
    private config: ApplicationConfigService,
    private certificateService: CertificateService,
    private backupsService: BackupsService,
    private hostsService: HostsService,
    private yamlService: YamlService,
  ) {}

  @Get()
  @ApiOkResponse({
    description: 'Return the list of host',
    type: [HostInformation],
  })
  async list(): Promise<HostInformation[]> {
    return Promise.all(
      (await this.hostsService.getHosts()).map(async (host) => {
        return new HostInformation(host, (await this.backupsService.getLastBackup(host)) ?? undefined);
      }),
    );
  }

  @Get(':name')
  @ApiOkResponse({
    description: 'Return the configuration of an host',
    type: HostConfiguration,
  })
  async get(@Param('name') name: string): Promise<HostConfiguration> {
    const host = await this.hostsService.getHost(name);
    if (!host) {
      throw new NotFoundException(`Can't find the host with name ${name}`);
    }
    return host;
  }

  @Get(':name/client')
  @ApiHeader({ name: 'content-type', required: false })
  @ApiProduces('application/zip', 'application/x-binary', 'text/plain')
  async downloadClient(
    @Param('name') name: string,
    @Res() res: Response,
    @Headers('content-type') type?: string,
  ): Promise<void> {
    let archive: archiver.Archiver;
    switch (type || 'application/zip') {
      case 'application/zip':
        archive = archiver.create('zip');
        break;
      case 'application/x-tar':
        archive = archiver.create('tar');
        break;
      default:
        throw new UnsupportedMediaTypeException(`Unsupported media type: ${type}`);
    }

    res.attachment(`client.zip`);
    archive.pipe(res);

    // Générer les fichiers pour le host
    await this.certificateService.generateHostCertificate(name);

    const host = await this.hostsService.getHost(name);

    archive.file(join(this.config.certificatePath, 'rootCA.pem'), { name: 'rootCA.pem' });
    archive.file(join(this.config.certificatePath, 'public_key.pem'), { name: 'public_key.pem' });
    archive.file(join(this.config.certificatePath, `${name}.pem`), { name: `${name}.pem` });
    archive.file(join(this.config.certificatePath, `${name}.key`), { name: `${name}.key` });
    archive.append(await this.yamlService.writeBuffer({ hostname: name, password: host.password }), {
      name: `config.yaml`,
    });

    await archive.finalize();
  }
}
