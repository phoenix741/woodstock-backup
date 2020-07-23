import { BullModule, InjectQueue } from '@nestjs/bull';
import { Module, OnModuleInit } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { HttpAdapterHost } from '@nestjs/core';
import { GraphQLModule } from '@nestjs/graphql';
import { ServeStaticModule } from '@nestjs/serve-static';
import { Queue } from 'bull';
import { setQueues, UI } from 'bull-board';
import { PubSub } from 'graphql-subscriptions';

import { BackupsFilesController } from './backups/backups-files.controller';
import { BackupsFilesService } from './backups/backups-files.service';
import { BackupController } from './backups/backups.controller';
import { BackupsResolver } from './backups/backups.resolver';
import { BackupsService } from './backups/backups.service';
import { ApplicationConfigModule } from './config/application-config.module';
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
import { JobResolver } from './queue/job.resolver';
import { QueueController } from './queue/queue.controller';
import { QueueResolver } from './queue/queue.resolver';
import { QueueService } from './queue/queue.service';
import { SchedulerConfigService } from './scheduler/scheduler-config.service';
import { SchedulerConsumer } from './scheduler/scheduler.consumer';
import { SchedulerService } from './scheduler/scheduler.service';
import { ServeStaticService } from './server/serve-static.service';
import { ServerController } from './server/server.controller';
import { ServerResolver } from './server/server.resolver';
import { ToolsService } from './server/tools.service';
import { BackupQuotaResolver } from './stats/backup-quota.resolver';
import { StatsConsumer } from './stats/stats.consumer';
import { StatsResolver } from './stats/stats.resolver';
import { StatsService } from './stats/stats.service';
import { TimestampBackupQuotaResolver } from './stats/timestamp-backup-quota.resolver';
import { BtrfsService } from './storage/btrfs/btrfs.service';
import { HostConsumer } from './tasks/host.consumer';
import { TasksService } from './tasks/tasks.service';
import { HostConsumerUtilService } from './utils/host-consumer-util.service';
import { LockService } from './utils/lock.service';
import { SharePathService } from './utils/share-path.service';
import { YamlService } from './utils/yaml.service';

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
      installSubscriptionHandlers: true,
      autoSchemaFile: true,
      buildSchemaOptions: {
        dateScalarMode: 'timestamp',
      },
    }),
    ServeStaticModule.forRootAsync({
      imports: [ApplicationConfigModule],
      useClass: ServeStaticService,
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
    ServerResolver,
    PingService,
    ApplicationLogger,
    BackupsResolver,
    BackupsService,
    BackupsFilesService,
    YamlService,
    LockService,
    SharePathService,
    JobResolver,
    QueueResolver,
    QueueService,
    ToolsService,
    HostConsumerUtilService,
    StatsConsumer,
    StatsService,
    StatsResolver,
    TimestampBackupQuotaResolver,
    BackupQuotaResolver,
    {
      provide: 'BACKUP_QUEUE_PUB_SUB',
      useValue: new PubSub(),
    },
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
