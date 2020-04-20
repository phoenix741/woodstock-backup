import { Injectable, Logger } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import * as fs from 'fs';
import * as yaml from 'js-yaml';
import * as mkdirp from 'mkdirp';

import { compact } from '../utils/lodash';
import { HostConfig } from './host-config.dto';
import { SchedulerConfigService } from '../scheduler/scheduler-config.service';

/**
 * Class used to manage configuration file for hosts.
 */
@Injectable()
export class HostsService {
  private logger = new Logger(HostsService.name);
  private _hosts: Array<HostConfig> = [];
  private configPath: string;
  private hostsPath: string;

  constructor(configService: ConfigService, private schedulerConfigService: SchedulerConfigService) {
    this.configPath = configService.get<string>('paths.configPath', '<defunct>');
    this.hostsPath = configService.get<string>('paths.configHostPath', '<defunct>');
  }

  async getHost(host: string): Promise<HostConfig | undefined> {
    return (await this.getHosts()).find(h => h.name === host);
  }

  /**
   * Get all hosts, and associated config file.
   */
  async getHosts(): Promise<HostConfig[]> {
    if (!this._hosts.length) {
      await this.loadHosts();
    }

    const schedulerConfig = await this.schedulerConfigService.getScheduler();
    return this._hosts.map(host => {
      host.schedule = Object.assign({}, host.schedule, schedulerConfig.defaultSchdule);
      return host;
    });
  }

  async addHost(config: HostConfig) {
    await this.loadHosts();

    this._hosts.push(config);

    await this.writeHosts();
  }

  /**
   * Load host from the file stored at this.hostsPath
   */
  private async loadHosts(): Promise<void> {
    this.logger.log(`Hosts.loadHosts: Read the file ${this.hostsPath}`);

    try {
      await mkdirp(this.configPath);

      const hostsFromStr = await fs.promises.readFile(this.hostsPath, 'utf8');
      this._hosts = yaml.safeLoad(hostsFromStr) || [];
    } catch (err) {
      this._hosts = [];
      this.logger.error(`Hosts.loadHosts: Can't read hosts files ${err.message}`);
    }
  }

  /**
   * Save all modification made on the config file in this.hostsPath
   */
  private async writeHosts(): Promise<void> {
    this.logger.log(`Hosts.writeHosts: Write the file ${this.hostsPath}`);

    try {
      await mkdirp(this.configPath);

      const hostsFromStr = yaml.safeDump(compact(this._hosts));
      await fs.promises.writeFile(this.hostsPath, hostsFromStr, 'utf-8');
    } catch (err) {
      this.logger.error(`Hosts.writeHosts: Can't write hosts files: ${err.message}`);
    }
  }
}
