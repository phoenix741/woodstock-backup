import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { InputOutputModule } from '../input-output/input-output.module.js';
import { ApplicationConfigService } from './application-config.service.js';
import { BackupsService } from './backups.service.js';
import { HostsService } from './hosts.service.js';
import { SchedulerConfigService } from './scheduler-config.service.js';

@Module({
  imports: [ConfigModule, InputOutputModule],
  providers: [ApplicationConfigService, BackupsService, HostsService, SchedulerConfigService],
  exports: [ApplicationConfigService, BackupsService, HostsService, SchedulerConfigService],
})
export class ApplicationConfigModule {}
