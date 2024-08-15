import {
  ClassSerializerInterceptor,
  Controller,
  Get,
  Headers,
  Logger,
  NotFoundException,
  Param,
  Query,
  Res,
  UnsupportedMediaTypeException,
  UseInterceptors,
} from '@nestjs/common';
import { ApiHeader, ApiOkResponse, ApiProduces, ApiQuery } from '@nestjs/swagger';
import { ApplicationConfigService, findNearestPackageJson, YamlService } from '@woodstock/shared';
import { HostConfiguration } from '@woodstock/shared';
import { CertificateService } from '@woodstock/shared';
import * as archiver from 'archiver';
import { Response } from 'express';
import { join } from 'path';
import { ClientType, HostInformation } from './hosts.dto.js';
import { BackupsService, HostsService } from '@woodstock/shared';
import { readFile } from 'fs/promises';

const CLIENT_URL_MAPPING: Record<ClientType, string> = {
  [ClientType.Windows]: 'ws_client_daemon.exe',
  [ClientType.Linux]: 'ws_client_daemon',
  [ClientType.LinuxLite]: 'ws_client_daemon_lite',
  [ClientType.None]: 'ws_client_daemon',
};

const CLIENT_ZIP_URL_MAPPING: Record<ClientType, string> = {
  [ClientType.Windows]: 'ws_client_daemon.exe',
  [ClientType.Linux]: 'ws_client_daemon',
  [ClientType.LinuxLite]: 'ws_client_daemon',
  [ClientType.None]: 'ws_client_daemon',
};

@UseInterceptors(ClassSerializerInterceptor)
@Controller('hosts')
export class HostController {
  #logger = new Logger(HostController.name);

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

  async #findVersion(): Promise<string> {
    // Get the version from the package.json
    const packageJson = await findNearestPackageJson();
    if (!packageJson) {
      throw new Error("Can't find the package.json");
    }

    const content = await readFile(packageJson, 'utf-8');
    const json = JSON.parse(content);

    return json.version;
  }

  async #findClient(clientType: ClientType, version: string): Promise<Buffer> {
    const clientName = CLIENT_URL_MAPPING[clientType];

    const clientUrl = `https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/releases/download/v${version}/${clientName}`;
    this.#logger.log(`Downloading client from ${clientUrl}`);

    const response = await fetch(clientUrl);

    const body = await response.arrayBuffer();

    return Buffer.from(body);
  }

  @Get(':name/client')
  @ApiHeader({ name: 'content-type', required: false })
  @ApiProduces('application/zip', 'application/x-binary', 'text/plain')
  @ApiQuery({ name: 'client', enum: ClientType, required: false })
  async downloadClient(
    @Param('name') name: string,
    @Res() res: Response,
    @Headers('content-type') type?: string,
    @Query('client') clientType: ClientType = ClientType.None,
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
    archive.file(join(this.config.certificatePath, `public_key.pem`), { name: `public_key.pem` });
    archive.file(join(this.config.certificatePath, `${name}_server.pem`), { name: `${name}_server.pem` });
    archive.file(join(this.config.certificatePath, `${name}_server.key`), { name: `${name}_server.key` });
    archive.append(await this.yamlService.writeBuffer({ hostname: name, password: host.password }), {
      name: `config.yaml`,
    });
    if (clientType !== ClientType.None) {
      const name = CLIENT_ZIP_URL_MAPPING[clientType];

      archive.append(await this.#findClient(clientType, await this.#findVersion()), { name, mode: 0o755 });
    }

    await archive.finalize();
  }
}
