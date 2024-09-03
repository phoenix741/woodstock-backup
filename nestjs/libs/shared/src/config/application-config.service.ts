import { Injectable } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import type { RedisOptions } from 'ioredis';
import { homedir } from 'os';
import { join } from 'path';
import { BackupContext, generateContext } from '@woodstock/shared-rs';

import { pick } from '../utils/objects.utils';

@Injectable()
export class ApplicationConfigService {
  #context: BackupContext;

  constructor(private configService: ConfigService) {
    this.#context = generateContext({
      backupPath: this.backupPath,
      certificatesPath: this.certificatePath,
      configPath: this.configPath,
      hostsPath: this.hostPath,
      logsPath: this.logPath,
      poolPath: this.poolPath,
      jobsPath: this.jobPath,
      logLevel: this.logLevel,
    });
  }

  get context(): BackupContext {
    return this.#context;
  }

  #isRoot() {
    return process.getuid && process.getuid() === 0;
  }

  get staticPath(): string {
    return this.configService.get('STATIC_PATH', join(__dirname, '..', '..', '..', 'client', 'dist'));
  }

  get clientPath(): string {
    return this.configService.get(
      'CLIENT_PATH',
      this.#isRoot() ? join(this.backupPath, 'client') : join(homedir(), '.woodstock'),
    );
  }

  get backupPath(): string {
    return this.configService.get('BACKUP_PATH', '/var/lib/woodstock');
  }

  get certificatePath(): string {
    return this.configService.get('CERTIFICATES_PATH', join(this.backupPath, 'certs'));
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

  get jobPath(): string {
    return this.configService.get('JOB_PATH', join(this.logPath, 'jobs'));
  }

  get redis(): RedisOptions {
    return {
      host: this.configService.get<string>('REDIS_HOST', 'localhost'),
      port: this.configService.get<number>('REDIS_PORT', 6379),
    };
  }

  get cacheTtl(): number {
    return this.configService.get<number>('CACHE_TTL', 24 * 3600) * 1000;
  }

  get logLevel(): string {
    return this.configService.get<string>('LOG_LEVEL', 'info');
  }

  toJSON(): Pick<
    this,
    | 'backupPath'
    | 'configPath'
    | 'configPathOfHosts'
    | 'configPathOfScheduler'
    | 'configPathOfTools'
    | 'certificatePath'
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
      'certificatePath',
      'hostPath',
      'logPath',
      'poolPath',
    );
  }
}
