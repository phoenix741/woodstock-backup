import { Injectable } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import * as Redis from 'ioredis';
import { join } from 'path';
import { PrometheusConnectionOptions } from 'prometheus-query';
import { pick } from '../utils';

@Injectable()
export class ApplicationConfigService {
  constructor(private configService: ConfigService) {}

  get staticPath(): string {
    return this.configService.get('STATIC_PATH', join(__dirname, '..', '..', '..', 'client', 'dist'));
  }

  get backupPath(): string {
    return this.configService.get('BACKUP_PATH', '/var/lib/woodstock');
  }

  get configPath(): string {
    return this.configService.get('CONFIG_PATH', join(this.backupPath, 'config'));
  }

  get statisticsPath(): string {
    return join(this.hostPath, 'statistics.yml');
  }

  get configPathOfHosts(): string {
    return join(this.configPath, 'hosts.yml');
  }

  get configPathOfScheduler(): string {
    return join(this.configPath, 'scheduler.yml');
  }

  get configPathOfTools(): string {
    return join(this.configPath, 'tools.yml');
  }

  get hostPath(): string {
    return this.configService.get('HOST_PATH', join(this.backupPath, 'hosts'));
  }

  get logPath(): string {
    return this.configService.get('LOG_PATH', join(this.backupPath, 'log'));
  }

  get poolPath(): string {
    return this.configService.get('POOL_PATH', join(this.backupPath, 'pool'));
  }

  get redis(): Redis.RedisOptions {
    return {
      host: this.configService.get<string>('REDIS_HOST', 'localhost'),
      port: this.configService.get<number>('REDIS_PORT', 6379),
    };
  }

  get logLevel(): string {
    return this.configService.get<string>('LOG_LEVEL', 'info');
  }

  get prometheus(): PrometheusConnectionOptions {
    return {
      endpoint: this.configService.get<string>('PROMETHEUS_ENDPOINT', 'http://localhost:9090'),
      baseURL: this.configService.get<string>('PROMETHEUS_BASE_URL', '/api/v1'),
    };
  }

  get prometheusInstanceFilter(): string {
    return this.configService.get<string>('PROMETHEUS_INSTANCE_FILTER', 'localhost:3000');
  }

  toJSON(): Pick<
    this,
    | 'backupPath'
    | 'configPath'
    | 'configPathOfHosts'
    | 'configPathOfScheduler'
    | 'configPathOfTools'
    | 'hostPath'
    | 'logPath'
    | 'poolPath'
  > {
    return pick(
      this,
      'backupPath',
      'configPath',
      'configPathOfHosts',
      'configPathOfScheduler',
      'configPathOfTools',
      'hostPath',
      'logPath',
      'poolPath',
    );
  }
}
