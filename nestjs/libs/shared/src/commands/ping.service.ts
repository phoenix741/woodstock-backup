import { Injectable, Logger } from '@nestjs/common';
import { grpcPing } from '@woodstock/shared-rs';
import { ApplicationConfigService } from '../config/application-config.service.js';
import { InformationToResolve } from './resolve.model.js';
import { ResolveService } from './resolve.service.js';

@Injectable()
export class PingService {
  private logger = new Logger(PingService.name);

  constructor(
    private config: ApplicationConfigService,
    private resolveService: ResolveService,
  ) {}

  async pingFromConfig(hostname: string, config?: InformationToResolve): Promise<string | undefined> {
    try {
      this.logger.debug(`Ping host ${hostname} from config`);
      const ip = await this.resolveService.resolveFromConfig(hostname, config);

      for (const address of ip) {
        this.logger.debug(`IP for the host ${hostname} is ${address}`);
        const result = await this.ping(address, hostname);
        this.logger.debug(`Ping of the host ${hostname} with IP ${address}: ${result}`);

        if (result) {
          return address;
        }
      }
    } catch (err) {
      this.logger.debug(`Can't find an ip for the host ${hostname}`);
    }
  }

  async ping(ip: string, hostname: string): Promise<boolean> {
    try {
      return await grpcPing(ip, hostname, this.config.context);
    } catch (err) {
      return false;
    }
  }
}
