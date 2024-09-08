import { BullModule } from '@nestjs/bullmq';
import { Module } from '@nestjs/common';
import {
  CoreBackupsService,
  CoreClientResolver,
  CoreFilesService,
  CoreHostsService,
  CorePoolService,
} from '@woodstock/shared-rs';
import { CertificateService } from './authentification';
import { JobService } from './backuping';
import { BackupsService, HostsService, LockService } from './backups';
import { PoolService } from './backups/pool.service';
import { ExecuteCommandService } from './commands/execute-command.service';
import { PingService } from './commands/ping.service';
import { ResolveService } from './commands/resolve.service';
import { ToolsService } from './commands/tools.service';
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
  ExecuteCommandService,
  FilesService,
  HostsService,
  JobService,
  LockService,
  PingService,
  PoolService,
  PoolStatisticsService,
  QueueTasksService,
  ResolveService,
  SchedulerConfigService,
  StatsInstantService,
  ToolsService,
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
      useFactory: (config: ApplicationConfigService) => new CoreHostsService(config.context),
      inject: [ApplicationConfigService],
    },
    {
      provide: CoreBackupsService,
      useFactory: (config: ApplicationConfigService) => new CoreBackupsService(config.context),
      inject: [ApplicationConfigService],
    },
    {
      provide: CoreFilesService,
      useFactory: (config: ApplicationConfigService) => new CoreFilesService(config.context),
      inject: [ApplicationConfigService],
    },
    {
      provide: CorePoolService,
      useFactory: (config: ApplicationConfigService) => new CorePoolService(config.context),
      inject: [ApplicationConfigService],
    },
    {
      provide: CoreClientResolver,
      useFactory: () => new CoreClientResolver(),
    },
  ],
  exports: [...providers, RegisteredQueue],
})
export class SharedModule {}
