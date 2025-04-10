import { Injectable } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import { getConfiguration, JsConfiguration } from '@woodstock/shared-rs';
import type { RedisOptions } from 'ioredis';
import { join } from 'path';

@Injectable()
export class ApplicationConfigService {
  #configuration: JsConfiguration;

  constructor(private configService: ConfigService) {
    this.#configuration = getConfiguration();
  }

  get context(): JsConfiguration {
    return this.#configuration;
  }

  get staticPath(): string {
    return this.configService.get('STATIC_PATH', join(__dirname, '..', '..', '..', 'client', 'dist'));
  }

  get backupPath(): string {
    return this.#configuration.path.backupPath;
  }

  get certificatePath(): string {
    return this.#configuration.path.certificatesPath;
  }

  get configPath(): string {
    return this.#configuration.path.configPath;
  }

  get configPathOfScheduler(): string {
    return this.#configuration.path.configPathScheduler;
  }

  get hostPath(): string {
    return this.#configuration.path.hostsPath;
  }

  get logPath(): string {
    return this.#configuration.path.logsPath;
  }

  get poolPath(): string {
    return this.#configuration.path.poolPath;
  }

  get jobPath(): string {
    return this.#configuration.path.jobsPath;
  }

  get redis(): RedisOptions {
    return this.#configuration.redis;
  }

  get cacheTtl(): number {
    return this.configService.get<number>('CACHE_TTL', 24 * 3600) * 1000;
  }

  get clientApiHostname(): string {
    return this.configService.get<string>('CLIENT_API_HOSTNAME', 'localhost');
  }

  get clientApiPort(): number {
    return parseInt(this.configService.get<string>('CLIENT_API_PORT', '8443'));
  }
}
