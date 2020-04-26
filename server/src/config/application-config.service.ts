import { Injectable } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import { join } from 'path';

@Injectable()
export class ApplicationConfigService {
  constructor(private configService: ConfigService) {}

  get backupPath(): string {
    return this.configService.get('BACKUP_PATH', '/var/lib/woodstock');
  }

  get configPath(): string {
    return this.configService.get('CONFIG_PATH', join(this.backupPath, 'config'));
  }

  get configPathOfHosts(): string {
    return join(this.configPath, 'hosts.yml');
  }

  get configPathOfScheduler(): string {
    return join(this.configPath, 'scheduler.yml');
  }

  get hostPath(): string {
    return this.configService.get('HOST_PATH', join(this.backupPath, 'hosts'));
  }

  get logPath(): string {
    return this.configService.get('LOG_PATH', join(this.backupPath, 'log'));
  }

  get redis() {
    return {
      host: this.configService.get<string>('REDIS_HOST', 'localhost'),
      port: this.configService.get<number>('REDIS_PORT', 6379),
    };
  }
}
