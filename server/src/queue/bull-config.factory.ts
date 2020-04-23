import { Injectable } from '@nestjs/common';
import { BullOptionsFactory, BullModuleOptions } from '@nestjs/bull';
import { ConfigService } from '@nestjs/config';

@Injectable()
export class BullConfigService implements BullOptionsFactory {
  constructor(private configService: ConfigService) {}

  createBullOptions(): BullModuleOptions {
    return {
      redis: {
        host: this.configService.get<string>('redis.host', 'localhost'),
        port: this.configService.get<number>('redis.port', 6379),
      },
      prefix: 'woodstock-backup',
    };
  }
}
