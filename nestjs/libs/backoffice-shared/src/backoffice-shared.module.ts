import { BullModule } from '@nestjs/bull';
import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { SharedModule } from '@woodstock/shared';
import { BullConfigService } from './config';
import { ApplicationConfigModule } from './config/application-config.module';
import {
  BackupsService,
  ExecuteCommandService,
  HostsService,
  LockService,
  PingService,
  PoolChunkRefCnt,
  PoolService,
  ResolveService,
  StatsService,
  ToolsService,
} from './services';
import { FilesService } from './services/files.service';
import { SchedulerConfigService } from './services/scheduler-config.service';

export const WoodstockQueueModule = [
  BullModule.registerQueue(
    {
      name: 'queue',
    },
    {
      name: 'schedule',
    },
    {
      name: 'stats',
    },
  ),
];

export const WoodstockBullModules = [
  BullModule.forRootAsync({
    imports: [ApplicationConfigModule],
    useClass: BullConfigService,
  }),
  ...WoodstockQueueModule,
];

const PROVIDERS = [
  BackupsService,
  FilesService,
  ExecuteCommandService,
  HostsService,
  LockService,
  PingService,
  PoolChunkRefCnt,
  PoolService,
  ResolveService,
  SchedulerConfigService,
  StatsService,
  ToolsService,
];

@Module({
  imports: [ConfigModule, ApplicationConfigModule, SharedModule, ...WoodstockQueueModule],
  providers: PROVIDERS,
  exports: PROVIDERS,
})
export class BackofficeSharedModule {}
