import { BullModule } from '@nestjs/bullmq';
import { Module } from '@nestjs/common';
import { CoreModule } from '@woodstock/core';
import { SharedModule } from '@woodstock/shared';
import { JobService } from './backuping';
import { BackupsService, HostsService, LockService } from './backups';
import { FilesService } from './files';
import { FsckService, PoolService } from './pool';
import { BullConfigService, RegisteredQueue } from './queue';
import { RefCntService } from './refcnt';
import { DiskStatisticsService, PoolStatisticsService, StatsInstantService } from './statistics';
import { QueueTasksService } from './tasks';

const providers = [
  DiskStatisticsService,
  FilesService,
  FsckService,
  PoolService,
  PoolStatisticsService,
  QueueTasksService,
  RefCntService,
  StatsInstantService,
  JobService,
  BackupsService,
  HostsService,
  LockService,
];

@Module({
  imports: [
    CoreModule,
    BullModule.forRootAsync({
      imports: [CoreModule],
      useClass: BullConfigService,
    }),
    RegisteredQueue,
    SharedModule,
  ],
  providers,
  exports: [...providers, RegisteredQueue],
})
export class ServerModule {}
