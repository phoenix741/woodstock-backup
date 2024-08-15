import { CACHE_MANAGER } from '@nestjs/cache-manager';
import { Inject, Injectable } from '@nestjs/common';
import { CoreHostsService, JsHostConfiguration, JsSchedule } from '@woodstock/shared-rs';
import { Cache } from 'cache-manager';
import { SchedulerConfigService } from '../config';

@Injectable()
export class HostsService {
  constructor(
    @Inject(CACHE_MANAGER) private cacheManager: Cache,
    private hostService: CoreHostsService,
    private schedulerConfigService: SchedulerConfigService,
  ) {}

  getHosts(): Promise<string[]> {
    return this.cacheManager.wrap('hosts', () => this.hostService.list());
  }

  getHost(hostname: string): Promise<JsHostConfiguration> {
    return this.cacheManager.wrap(`host-${hostname}`, () => this.hostService.get(hostname));
  }

  async getSchedule(hostname: string): Promise<JsSchedule> {
    const config = await this.getHost(hostname);

    let schedulerConfig = await this.schedulerConfigService.getScheduler();
    return Object.assign({}, schedulerConfig.defaultSchedule, config.schedule);
  }

  async invalidateHost(hostname: string): Promise<void> {
    await this.cacheManager.del(`host-${hostname}`);
    await this.cacheManager.del('hosts');
  }
}
