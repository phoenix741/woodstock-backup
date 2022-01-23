import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { GraphQLModule } from '@nestjs/graphql';
import { ServeStaticModule } from '@nestjs/serve-static';
import { ApplicationConfigModule, BackofficeSharedModule, WoodstockBullModules } from '@woodstock/backoffice-shared';
import { SharedModule } from '@woodstock/shared';
import { PubSub } from 'graphql-subscriptions';
import { BackupsFilesController } from './backups/backups-files.controller';
import { BackupsFilesService } from './backups/backups-files.service';
import { BackupController } from './backups/backups.controller';
import { BackupsResolver } from './backups/backups.resolver';
import { HostController } from './hosts/hosts.controller';
import { HostsResolver } from './hosts/hosts.resolver';
import { JobResolver } from './queue/job.resolver';
import { QueueController } from './queue/queue.controller';
import { QueueResolver } from './queue/queue.resolver';
import { QueueService } from './queue/queue.service';
import { SchedulerConsumer } from './scheduler/scheduler.consumer';
import { SchedulerService } from './scheduler/scheduler.service';
import { ServeStaticService } from './server/serve-static.service';
import { ServerController } from './server/server.controller';
import { ServerService } from './server/server.service';
import { BackupQuotaResolver } from './stats/backup-quota.resolver';
import { StatsResolver } from './stats/stats.resolver';
import { TimestampBackupQuotaResolver } from './stats/timestamp-backup-quota.resolver';
import { BigIntScalar } from './utils/bigint.scalar';

@Module({
  imports: [
    ApplicationConfigModule,
    BackofficeSharedModule,
    ...WoodstockBullModules,
    ConfigModule.forRoot(),
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
    BackupController,
    BackupQuotaResolver,
    BackupsFilesController,
    BackupsFilesService,
    BackupsResolver,
    BigIntScalar,
    HostController,
    HostsResolver,
    JobResolver,
    QueueController,
    QueueResolver,
    QueueService,
    SchedulerConsumer,
    SchedulerService,
    ServerController,
    ServerService,
    ServeStaticService,
    StatsResolver,
    TimestampBackupQuotaResolver,
    {
      provide: 'BACKUP_QUEUE_PUB_SUB',
      useValue: new PubSub(),
    },
  ],
})
export class AppModule {}
