import { Injectable, Logger } from '@nestjs/common';
import { ExecuteCommandService } from './execute-command.service.js';
import { InformationToResolve } from './resolve.model.js';
import { ResolveService } from './resolve.service.js';
import { CommandParameters } from './tools.service.js';

@Injectable()
export class PingService {
  private logger = new Logger(PingService.name);

  constructor(
    private executeCommandService: ExecuteCommandService,
    private resolveService: ResolveService,
  ) {}

  async pingFromConfig(hostname: string, config: InformationToResolve): Promise<boolean> {
    try {
      this.logger.debug(`Ping host ${hostname} from config`);
      const ip = await this.resolveService.resolveFromConfig(hostname, config);
      this.logger.debug(`IP for the host ${hostname} is ${ip}`);
      const result = await this.ping({ ip, hostname });
      this.logger.debug(`Ping of the host ${hostname} with IP ${ip}: ${result}`);
      return result;
    } catch (err) {
      this.logger.debug(`Can't find an ip for the host ${hostname}`);
      return false;
    }
  }

  // TODO: IPv6
  async ping(params: CommandParameters): Promise<boolean> {
    try {
      await this.executeCommandService.executeTool('ping', params);
      return true;
    } catch (err) {
      return false;
    }
  }
}
