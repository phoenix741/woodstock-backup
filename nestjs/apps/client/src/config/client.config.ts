import { Injectable, Logger, OnModuleInit } from '@nestjs/common';
import { ApplicationConfigService, YamlService } from '@woodstock/shared';
import { hostname } from 'os';
import { join } from 'path';

export interface ClientConfigFile {
  hostname: string;
  bind: string;
  password: string;
}

const DEFAULT_CONFIG: ClientConfigFile = {
  hostname: hostname(),
  bind: '0.0.0.0:3657',
  password: '',
};

export default DEFAULT_CONFIG;

@Injectable()
export class ClientConfigService implements OnModuleInit {
  #logger = new Logger(ClientConfigService.name);
  #config: ClientConfigFile;

  constructor(private configService: ApplicationConfigService, private yamlService: YamlService) {}

  async onModuleInit() {
    this.#config = Object.assign(
      {},
      DEFAULT_CONFIG,
      await this.yamlService.loadFile(join(this.configService.clientPath, 'config.yaml'), DEFAULT_CONFIG),
    );
    this.#logger.log(`Starting client on ${this.config.hostname} (listening on port ${this.config.bind})`);
  }

  get config() {
    return this.#config;
  }

  get rootCA() {
    return join(this.configService.clientPath, 'rootCA.pem');
  }

  get privateKey() {
    return join(this.configService.clientPath, `${this.config.hostname}.key`);
  }

  get publicKey() {
    return join(this.configService.clientPath, `${this.config.hostname}.pem`);
  }
}
