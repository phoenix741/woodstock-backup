import { BullModule } from '@nestjs/bull';
import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { ApplicationConfigModule, BullConfigService } from './config';
import { FileBrowserService } from './file/file-browser.service';
import { FileReader } from './file/file-reader.service';
import { ManifestService } from './manifest/manifest.service';
import { RefCntService } from './refcnt';
import {
  BackupsService,
  ExecuteCommandService,
  FilesService,
  HostsService,
  LockService,
  PingService,
  PoolChunkRefCnt,
  PoolService,
  ProtobufService,
  ResolveService,
  SchedulerConfigService,
  ToolsService,
  YamlService,
} from './services';
import { PoolStatisticsService, StatsInstantService, DiskStatisticsService } from './statistics';

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
  ExecuteCommandService,
  FilesService,
  HostsService,
  LockService,
  PingService,
  PoolChunkRefCnt,
  PoolService,
  PoolStatisticsService,
  ResolveService,
  SchedulerConfigService,
  StatsInstantService,
  DiskStatisticsService,
  ToolsService,
  FileReader,
  FileBrowserService,
  ManifestService,
  YamlService,
  ProtobufService,
  RefCntService,
];

@Module({
  imports: [ConfigModule, ApplicationConfigModule, SharedModule, ...WoodstockQueueModule],
  providers: PROVIDERS,
  exports: PROVIDERS,
})
export class SharedModule {}
