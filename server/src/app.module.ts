import { BullModule, InjectQueue } from '@nestjs/bull';
import { Module, OnModuleInit } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { HttpAdapterHost } from '@nestjs/core';
import { GraphQLModule } from '@nestjs/graphql';
import { Queue } from 'bull';
import { setQueues, UI } from 'bull-board';

import { BackupsFilesController } from './backups/backups-files.controller';
import { BackupsFilesService } from './backups/backups-files.service';
import { BackupController } from './backups/backups.controller';
import { ApplicationConfigService } from './config/application-config.service';
import { HostController } from './hosts/hosts.controller';
import { HostsResolver } from './hosts/hosts.resolver';
import { HostsService } from './hosts/hosts.service';
import { ApplicationLogger } from './logger/ApplicationLogger.logger';
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
import { YamlService } from './utils/yaml.service';
import { ApplicationConfigModule } from './config/application-config.module';

@Module({
  imports: [
    ConfigModule.forRoot(),
    ApplicationConfigModule,
    BullModule.registerQueueAsync(
      {
        name: 'queue',
        imports: [ApplicationConfigModule],
        useClass: BullConfigService,
      },
      {
        name: 'schedule',
        imports: [ApplicationConfigModule],
        useClass: BullConfigService,
      },
    ),
    GraphQLModule.forRoot({
      autoSchemaFile: true,
    }),
  ],
  controllers: [QueueController, BackupController, HostController, ServerController, BackupsFilesController],
  providers: [
    ApplicationConfigService,
    TasksService,
    ResolveService,
    ExecuteCommandService,
    RSyncCommandService,
    HostsService,
    HostsResolver,
    HostConsumer,
    BtrfsService,
    SchedulerService,
    SchedulerConfigService,
    SchedulerConsumer,
    PingService,
    ApplicationLogger,
    BackupsFilesService,
    YamlService,
  ],
})
export class AppModule implements OnModuleInit {
  constructor(
    private adapterHost: HttpAdapterHost,
    @InjectQueue('queue') private queue: Queue,
    @InjectQueue('schedule') private schedule: Queue,
  ) {}

  onModuleInit() {
    setQueues([this.queue, this.schedule]);
    this.adapterHost.httpAdapter.getInstance().use('/admin', UI);
  }
}
