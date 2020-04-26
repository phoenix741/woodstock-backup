import { Injectable, Logger } from '@nestjs/common';

import { ApplicationConfigService } from '../config/application-config.service';
import { YamlService } from '../utils/yaml.service';
import { ApplicationScheduler } from './scheduler.dto';

const DEFAULT_SCHEDULER = new ApplicationScheduler();

/**
 * Class used to manage configuration file for hosts.
 */
@Injectable()
export class SchedulerConfigService {
  private logger = new Logger(SchedulerConfigService.name);

  constructor(private configService: ApplicationConfigService, private yamlService: YamlService) {}

  /**
   * Get all hosts, and associated config file.
   */
  async getScheduler(): Promise<ApplicationScheduler> {
    return await this.yamlService.loadFile(this.configService.configPathOfScheduler, DEFAULT_SCHEDULER);
  }

  async setScheduler(config: ApplicationScheduler) {
    await this.yamlService.writeFile(this.configService.configPathOfScheduler, config);
  }
}
