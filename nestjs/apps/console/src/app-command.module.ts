import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import {
  ApplicationConfigModule,
  CoreModule,
  InputOutputModule,
  MaintenanceModule,
  PoolModule,
  QueueModule,
  ScannerModule,
} from '@woodstock/shared';
import { QueueTasksModule } from '@woodstock/shared/tasks/queue-tasks.module.js';
import { ConsoleModule } from 'nestjs-console';
import { BackupsCommand } from './backups.command.js';
import { BrowserCommand } from './browser.command.js';
import { GlobalModule } from './global.module.js';
import { PoolCommand } from './pool.command.js';
import { ProtobufCommand } from './protobuf.command.js';
import { BackupQueueStatus, RefcntQueueStatus } from './queue-status.service.js';
import { StatsCommand } from './stats.command.js';

@Module({
  imports: [
    ConfigModule.forRoot(),
    GlobalModule,
    CoreModule,
    ConsoleModule,
    ApplicationConfigModule,
    ScannerModule,
    MaintenanceModule,
    PoolModule,
    InputOutputModule,
    QueueModule,
    QueueTasksModule,
  ],
  providers: [
    RefcntQueueStatus,
    BackupQueueStatus,
    BackupsCommand,
    BrowserCommand,
    PoolCommand,
    ProtobufCommand,
    StatsCommand,
  ],
})
export class AppCommandModule {}
