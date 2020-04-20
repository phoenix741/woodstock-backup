import { BullModule } from '@nestjs/bull';
import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';

import { BackupController } from './backups/backups.controller';
import configuration from './config/configuration';
import { HostController } from './hosts/hosts.controller';
import { HostsService } from './hosts/hosts.service';
import { PingService } from './network/ping';
import { ResolveService } from './network/resolve';
import { ExecuteCommandService } from './operation/execute-command.service';
import { RSyncCommandService } from './operation/rsync-command.service';
import { BullConfigService } from './queue/bull-config.factory';
import { QueueController } from './queue/queue.controller';
import { SchedulerConfigService } from './scheduler/scheduler-config.service';
import { SchedulerConsumer } from './scheduler/scheduler.consumer';
import { SchedulerService } from './scheduler/scheduler.service';
import { ServerController } from './server/server.controller';
import { BtrfsService } from './storage/btrfs/btrfs.service';
import { HostConsumer } from './tasks/host.consumer';
import { TasksService } from './tasks/tasks.service';

@Module({
  imports: [
    ConfigModule.forRoot({
      load: [configuration],
    }),
    BullModule.registerQueueAsync(
      {
        name: 'queue',
        imports: [ConfigModule],
        useClass: BullConfigService,
      },
      {
        name: 'schedule',
        imports: [ConfigModule],
        useClass: BullConfigService,
      },
    ),
  ],
  controllers: [QueueController, BackupController, HostController, ServerController],
  providers: [TasksService, ResolveService, ExecuteCommandService, RSyncCommandService, HostsService, HostConsumer, BtrfsService, SchedulerService, SchedulerConfigService, SchedulerConsumer, PingService],
})
export class AppModule {}
