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
import { CertificateService } from './services/auth/certificate.service';
import { EncryptionService } from './services/auth/encryption.service';
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
  CertificateService,
  DiskStatisticsService,
  EncryptionService,
  ExecuteCommandService,
  FileBrowserService,
  FileReader,
  FilesService,
  HostsService,
  LockService,
  ManifestService,
  PingService,
  PoolChunkRefCnt,
  PoolService,
  PoolStatisticsService,
  ProtobufService,
  RefCntService,
  ResolveService,
  SchedulerConfigService,
  StatsInstantService,
  ToolsService,
  YamlService,
];

@Module({
  imports: [ConfigModule, ApplicationConfigModule, SharedModule, ...WoodstockQueueModule],
  providers: PROVIDERS,
  exports: PROVIDERS,
})
export class SharedModule {}
