import { Injectable } from '@nestjs/common';
import { CoreHostsService, JsHostConfiguration } from '@woodstock/shared-rs';

@Injectable()
export class HostsService {
  constructor(private hostService: CoreHostsService) {}

  getHosts(): Promise<string[]> {
    return this.hostService.list();
  }

  getHost(hostname: string): Promise<JsHostConfiguration> {
    return this.hostService.get(hostname);
  }
}
