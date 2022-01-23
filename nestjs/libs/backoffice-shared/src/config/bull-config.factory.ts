import { SharedBullConfigurationFactory } from '@nestjs/bull';
import { Injectable } from '@nestjs/common';
import Bull from 'bull';
import { ApplicationConfigService } from './application-config.service';

@Injectable()
export class BullConfigService implements SharedBullConfigurationFactory {
  constructor(private configService: ApplicationConfigService) {}

  createSharedConfiguration(): Bull.QueueOptions {
    return {
      redis: this.configService.redis,
      prefix: 'woodstock-backup',
    };
  }
}
