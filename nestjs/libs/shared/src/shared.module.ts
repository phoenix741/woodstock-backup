import { BullModule } from '@nestjs/bullmq';
import { Module } from '@nestjs/common';
import { CoreBackupsService, CoreClientResolver, CoreFilesService, CoreHostsService } from '@woodstock/shared-rs';
import { CertificateService } from './authentification';
import { JobService } from './backuping';
import { BackupsService, HostsService } from './backups';
import { PingService } from './commands/ping.service';
import { ResolveService } from './commands/resolve.service';
import { ApplicationConfigService, SchedulerConfigService } from './config';
import { FilesService } from './files';
import { BullConfigService, RegisteredQueue } from './queue';
import { YamlService } from './services';
import { DiskStatisticsService, PoolStatisticsService, StatsInstantService } from './statistics';
import { QueueTasksService } from './tasks';

@Module({
  providers: [ApplicationConfigService],
  exports: [ApplicationConfigService],
})
export class ConfigProviderModule {}

const providers = [
  BackupsService,
  CertificateService,
  DiskStatisticsService,
  FilesService,
  HostsService,
  JobService,
  PingService,
  PoolStatisticsService,
  QueueTasksService,
  ResolveService,
  SchedulerConfigService,
  StatsInstantService,
  YamlService,
];

@Module({
  imports: [
    ConfigProviderModule,
    BullModule.forRootAsync({
      useClass: BullConfigService,
      imports: [ConfigProviderModule],
    }),
    RegisteredQueue,
  ],
  providers: [
    ...providers,
    {
      provide: CoreHostsService,
      useFactory: () => new CoreHostsService(),
    },
    {
      provide: CoreBackupsService,
      useFactory: () => new CoreBackupsService(),
    },
    {
      provide: CoreFilesService,
      useFactory: () => new CoreFilesService(),
    },
    {
      provide: CoreClientResolver,
      useFactory: () => new CoreClientResolver(),
    },
  ],
  exports: [...providers, RegisteredQueue],
})
export class SharedModule {}
