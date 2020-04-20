import { Injectable, Logger } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import * as fs from 'fs';
import * as yaml from 'js-yaml';
import * as mkdirp from 'mkdirp';

import { ApplicationScheduler } from './scheduler.dto';

/**
 * Class used to manage configuration file for hosts.
 */
@Injectable()
export class SchedulerConfigService {
  private logger = new Logger(SchedulerConfigService.name);
  private _scheduler?: ApplicationScheduler;
  private configPath: string;
  private schedulerPath: string;

  constructor(configService: ConfigService) {
    this.configPath = configService.get<string>('paths.configPath', '<defunct>');
    this.schedulerPath = configService.get<string>('paths.schedulerPath', '<defunct>');
  }

  /**
   * Get all hosts, and associated config file.
   */
  async getScheduler(): Promise<ApplicationScheduler> {
    if (!this._scheduler) {
      this._scheduler = await this.loadConfig();
    }

    return this._scheduler;
  }

  async setScheduler(config: ApplicationScheduler) {
    await this.loadConfig();

    this._scheduler = config;

    await this.writeConfig();
  }

  /**
   * Load host from the file stored at this.hostsPath
   */
  private async loadConfig(): Promise<ApplicationScheduler> {
    this.logger.debug(`SchedulerService.loadConfig: Read the file ${this.schedulerPath}`);

    try {
      await mkdirp(this.configPath);

      const schedulerStr = await fs.promises.readFile(this.schedulerPath, 'utf8');
      return yaml.safeLoad(schedulerStr) || [];
    } catch (err) {
      this.logger.warn(`SchedulerService.loadConfig: Can't read hosts files ${err.message}`);
      return new ApplicationScheduler();
    }
  }

  /**
   * Save all modification made on the config file in this.hostsPath
   */
  private async writeConfig(): Promise<void> {
    this.logger.log(`SchedulerService.writeConfig: Write the file ${this.schedulerPath}`);

    await mkdirp(this.configPath);

    const schedulerStr = yaml.safeDump(this._scheduler);
    await fs.promises.writeFile(this.schedulerPath, schedulerStr, 'utf-8');
  }
}
