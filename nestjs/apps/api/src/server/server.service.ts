import { Injectable, Logger } from '@nestjs/common';
import { ApplicationConfigService, BackupsService, findNearestPackageJson, HostsService } from '@woodstock/shared';
import { mkdir, readFile } from 'node:fs/promises';
import { hostname, platform, uptime } from 'node:os';
import { ServerInformations } from './server.dto.js';

@Injectable()
export class ServerService {
  #logger = new Logger(ServerService.name);

  constructor(
    private applicationConfig: ApplicationConfigService,
    private hostsService: HostsService,
    private backupsService: BackupsService,
  ) {}

  async clearCache(): Promise<void> {
    this.#logger.log('Clear the cache of the server');
    await this.hostsService.invalidateHosts();
    for (const host of await this.hostsService.getHosts()) {
      await this.backupsService.invalidateBackups(host);
    }
  }

  async getInformations(): Promise<ServerInformations> {
    // Get the woodstock version from package.json
    const packageJsonPath = await findNearestPackageJson();
    const packageJson = packageJsonPath ? JSON.parse(await readFile(packageJsonPath, 'utf-8')) : undefined;

    return {
      hostname: hostname(),
      platform: platform(),
      uptime: uptime(),
      woodstockVersion: packageJson?.version,
    };
  }

  async initialize(): Promise<void> {
    await mkdir(this.applicationConfig.poolPath, { recursive: true });
    await mkdir(this.applicationConfig.backupPath, { recursive: true });
    await mkdir(this.applicationConfig.hostPath, { recursive: true });
    await mkdir(this.applicationConfig.configPath, { recursive: true });
    await mkdir(this.applicationConfig.logPath, { recursive: true });
  }
}
