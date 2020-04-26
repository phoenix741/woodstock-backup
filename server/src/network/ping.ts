import { Injectable, Logger } from '@nestjs/common';
import * as shell from 'shelljs';

import { HostConfiguration } from '../hosts/host-configuration.dto';
import { ResolveService } from './resolve';

@Injectable()
export class PingService {
  private logger = new Logger(PingService.name);

  constructor(private resolveService: ResolveService) {}

  async pingFromConfig(hostname: string, config: HostConfiguration): Promise<boolean> {
    try {
      this.logger.debug(`Ping host ${hostname} from config`);
      const ip = await this.resolveService.resolveFromConfig(hostname, config);
      this.logger.debug(`IP for the host ${hostname} is ${ip}`);
      const result = await this.ping(ip);
      this.logger.debug(`Ping of the host ${hostname} with IP ${ip}: ${result}`);
      return result;
    } catch (err) {
      this.logger.debug(`Can't find an ip for the host ${hostname}`);
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
