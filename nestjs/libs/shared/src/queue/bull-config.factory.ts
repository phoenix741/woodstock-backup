import { SharedBullConfigurationFactory } from '@nestjs/bullmq';
import { Injectable } from '@nestjs/common';
import Bull from 'bullmq';
import { ApplicationConfigService } from '../config';

@Injectable()
export class BullConfigService implements SharedBullConfigurationFactory {
  constructor(private configService: ApplicationConfigService) {}

  createSharedConfiguration(): Bull.QueueOptions {
    return {
      connection: this.configService.redis,
      prefix: 'woodstock-backup',
    };
  }
}
