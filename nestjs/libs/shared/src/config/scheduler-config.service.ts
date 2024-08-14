import { Injectable } from '@nestjs/common';
import { YamlService } from '../services/yaml.service';
import { ApplicationConfigService } from './application-config.service';
import { ApplicationScheduler, Schedule } from './scheduler.dto';

export const DEFAULT_SCHEDULER = new ApplicationScheduler({
  wakeupSchedule: '0 0 * * *',
  nightlySchedule: '0 0 * * *',
  defaultSchedule: new Schedule({
    activated: true,
    backupPeriod: 86400,
    backupToKeep: {
      hourly: 24,
      daily: 7,
      weekly: 4,
      monthly: 12,
      yearly: 1,
    },
  }),
});

/**
 * Class used to manage configuration file for hosts.
 */
@Injectable()
export class SchedulerConfigService {
  constructor(
    private configService: ApplicationConfigService,
    private yamlService: YamlService,
  ) {}

  /**
   * Get all hosts, and associated config file.
   */
  async getScheduler(): Promise<ApplicationScheduler> {
    return await this.yamlService.loadFile(this.configService.configPathOfScheduler, DEFAULT_SCHEDULER);
  }

  async setScheduler(config: ApplicationScheduler): Promise<void> {
    await this.yamlService.writeFile(this.configService.configPathOfScheduler, config);
  }
}
