import { Injectable } from '@nestjs/common';
import * as shell from 'shelljs';

import { HostConfig } from '../hosts/host-config.dto';
import { ResolveService } from './resolve';

@Injectable()
export class PingService {
  constructor(private resolveService: ResolveService) {}

  async pingFromConfig(config: HostConfig): Promise<boolean> {
    try {
      const ip = await this.resolveService.resolveFromConfig(config);
      return await this.ping(ip);
    } catch (err) {
      return false;
    }
  }

  // FIXME: IPv6
  async ping(ip: string): Promise<boolean> {
    return new Promise(resolve => {
      shell.exec(`ping -c 1 ${ip}`, { silent: true }, code => resolve(code === 0));
    });
  }
}
