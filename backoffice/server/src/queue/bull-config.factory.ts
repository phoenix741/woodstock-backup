import { BullModuleOptions, BullOptionsFactory } from '@nestjs/bull';
import { Injectable } from '@nestjs/common';

import { ApplicationConfigService } from '../config/application-config.service';

@Injectable()
export class BullConfigService implements BullOptionsFactory {
  constructor(private configService: ApplicationConfigService) {}

  createBullOptions(): BullModuleOptions {
    return {
      redis: this.configService.redis,
      prefix: 'woodstock-backup',
      settings: {
        stalledInterval: 0, // FIXME: Correction on big file required
      },
    };
  }
}
