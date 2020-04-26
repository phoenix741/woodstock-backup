import { BadRequestException, Injectable, NotFoundException } from '@nestjs/common';
import { join } from 'path';

import { ApplicationConfigService } from '../config/application-config.service';
import { YamlService } from '../utils/yaml.service';
import { HostConfiguration } from './host-configuration.dto';

/**
 * Class used to manage configuration file for hosts.
 */
@Injectable()
export class HostsService {
  constructor(private configService: ApplicationConfigService, private yamlService: YamlService) {}

  /**
   * Get the configuration of an host
   * @param host host name
   */
  async getHostConfiguration(host: string): Promise<HostConfiguration> {
    const hosts = await this.getHosts();
    if (!hosts.includes(host)) {
      throw new NotFoundException(`Can't find configuration for the host with name ${host}`);
    }

    return await this.yamlService.loadFile(this.getHostFile(host), new HostConfiguration());
  }

  /**
   * Create/Update the configuration of an host
   */
  async updateHostConfiguration(host: string, config: HostConfiguration) {
    const hosts = await this.getHosts();
    if (!hosts.includes(host)) {
      throw new NotFoundException(`Can't find configuration for the host with name ${host}`);
    }

    await this.yamlService.writeFile(this.getHostFile(host), config);
  }

  /**
   * Get all hosts names.
   */
  async getHosts(): Promise<string[]> {
    return this.yamlService.loadFile(this.configService.configPathOfHosts, []);
  }

  /**
   * Add an host in the configuration file
   *
   * @param host the hostname
   */
  async addHost(host: string) {
    if (host === 'hosts') {
      throw new BadRequestException(`Host ${host} is an invalid name`);
    }

    const hosts = new Set(await this.getHosts());
    hosts.add(host);

    await this.yamlService.writeFile(this.configService.configPathOfHosts, Array.from(hosts));
  }

  private getHostFile(host: string): string {
    return join(this.configService.configPath, `${host}.yml`);
  }
}
