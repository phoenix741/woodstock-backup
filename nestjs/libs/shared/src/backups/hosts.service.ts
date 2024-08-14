import { CACHE_MANAGER } from '@nestjs/cache-manager';
import { Inject, Injectable } from '@nestjs/common';
import { CoreHostsService, JsHostConfiguration } from '@woodstock/shared-rs';
import { Cache } from 'cache-manager';

@Injectable()
export class HostsService {
  constructor(
    @Inject(CACHE_MANAGER) private cacheManager: Cache,
    private hostService: CoreHostsService,
  ) {}

  getHosts(): Promise<string[]> {
    return this.cacheManager.wrap('hosts', () => this.hostService.list());
  }

  getHost(hostname: string): Promise<JsHostConfiguration> {
    return this.cacheManager.wrap(`host-${hostname}`, () => this.hostService.get(hostname));
  }

  async invalidateHost(hostname: string): Promise<void> {
    await this.cacheManager.del(`host-${hostname}`);
    await this.cacheManager.del('hosts');
  }
}
