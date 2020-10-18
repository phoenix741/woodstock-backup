import { BullModule } from '@nestjs/bull';
import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { GraphQLModule } from '@nestjs/graphql';
import { ServeStaticModule } from '@nestjs/serve-static';
import { SharedModule } from '@woodstock/shared';
import { PubSub } from 'graphql-subscriptions';
import { BackupsFilesController } from './backups/backups-files.controller';
import { BackupsFilesService } from './backups/backups-files.service';
import { BackupsGrpc } from './backups/backups-grpc.service';
import { BackupController } from './backups/backups.controller';
import { BackupsResolver } from './backups/backups.resolver';
import { BackupsService } from './backups/backups.service';
import { PoolService } from './backups/pool/pool.service';
import { ApplicationConfigModule } from './config/application-config.module';
import { ApplicationConfigService } from './config/application-config.service';
import { HostController } from './hosts/hosts.controller';
import { HostsResolver } from './hosts/hosts.resolver';
import { HostsService } from './hosts/hosts.service';
import { ApplicationLogger } from './logger/ApplicationLogger.logger';
import { PingService } from './network/ping';
import { ResolveService } from './network/resolve';
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
import { ServerService } from './server/server.service';
import { ToolsService } from './server/tools.service';
import { BackupQuotaResolver } from './stats/backup-quota.resolver';
import { StatsConsumer } from './stats/stats.consumer';
import { StatsResolver } from './stats/stats.resolver';
import { StatsService } from './stats/stats.service';
import { TimestampBackupQuotaResolver } from './stats/timestamp-backup-quota.resolver';
import { HostConsumer } from './tasks/host.consumer';
import { TasksService } from './tasks/tasks.service';
import { BigIntScalar } from './utils/bigint.scalar';
import { ExecuteCommandService } from './utils/execute-command.service';
import { HostConsumerUtilService } from './utils/host-consumer-util.service';
import { LockService } from './utils/lock.service';
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
    SharedModule,
  ],
  controllers: [QueueController, BackupController, HostController, ServerController, BackupsFilesController],
  providers: [
    ApplicationConfigService,
    ApplicationLogger,
    BackupQuotaResolver,
    BackupsFilesService,
    BackupsGrpc,
    BackupsResolver,
    BackupsService,
    BigIntScalar,
    ExecuteCommandService,
    HostConsumer,
    HostConsumerUtilService,
    HostsResolver,
    HostsService,
    JobResolver,
    LockService,
    PingService,
    PoolService,
    QueueResolver,
    QueueService,
    ResolveService,
    SchedulerConfigService,
    SchedulerConsumer,
    SchedulerService,
    ServerService,
    StatsConsumer,
    StatsResolver,
    StatsService,
    TasksService,
    TimestampBackupQuotaResolver,
    ToolsService,
    YamlService,
    {
      provide: 'BACKUP_QUEUE_PUB_SUB',
      useValue: new PubSub(),
    },
  ],
})
export class AppModule {}
