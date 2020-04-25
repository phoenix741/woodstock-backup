import { Injectable, Logger } from '@nestjs/common';
import * as shell from 'shelljs';

import { HostConfig } from '../hosts/host-config.dto';
import { ResolveService } from './resolve';

@Injectable()
export class PingService {
  private logger = new Logger(PingService.name);

  constructor(private resolveService: ResolveService) {}

  async pingFromConfig(config: HostConfig): Promise<boolean> {
    try {
      this.logger.debug(`Ping host ${config.name} from config`);
      const ip = await this.resolveService.resolveFromConfig(config);
      this.logger.debug(`IP for the host ${config.name} is ${ip}`);
      const result = await this.ping(ip);
      this.logger.debug(`Ping of the host ${config.name} with IP ${ip}: ${result}`);
      return result;
    } catch (err) {
      this.logger.debug(`Can't find an ip for the host ${config.name}`);
      return false;
    }
  }

  // FIXME: IPv6
  async ping(ip: string): Promise<boolean> {
    return new Promise(resolve => {
      const pingCommand = `ping -c 1 ${ip}`;
      this.logger.debug(`Execute ping command: ${pingCommand}`);
      shell.exec(pingCommand, { silent: true }, code => resolve(code === 0));
    });
  }
}
