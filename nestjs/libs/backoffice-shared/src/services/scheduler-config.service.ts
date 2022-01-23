import { Injectable } from '@nestjs/common';
import { YamlService } from '@woodstock/shared';
import { ApplicationConfigService } from '../config/application-config.service';
import { ApplicationScheduler, Schedule } from '../models/scheduler.dto';

const DEFAULT_SCHEDULER = new ApplicationScheduler({
  wakeupSchedule: '0 0 * * *',
  nightlySchedule: '0 0 * * *',
  defaultSchedule: new Schedule(),
});

/**
 * Class used to manage configuration file for hosts.
 */
@Injectable()
export class SchedulerConfigService {
  constructor(private configService: ApplicationConfigService, private yamlService: YamlService) {}

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
