import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { GraphQLModule } from '@nestjs/graphql';
import { ServeStaticModule } from '@nestjs/serve-static';
import { ApplicationConfigModule, SharedModule, WoodstockBullModules } from '@woodstock/shared';
import { PubSub } from 'graphql-subscriptions';
import { BackupsFilesController } from './backups/backups-files.controller';
import { BackupsFilesService } from './backups/backups-files.service';
import { BackupController } from './backups/backups.controller';
import { BackupsResolver } from './backups/backups.resolver';
import { GlobalModule } from './global.module';
import { HostController } from './hosts/hosts.controller';
import { HostsResolver } from './hosts/hosts.resolver';
import { JobResolver } from './queue/job.resolver';
import { QueueController } from './queue/queue.controller';
import { QueueResolver } from './queue/queue.resolver';
import { QueueService } from './queue/queue.service';
import { ServeStaticService } from './server/serve-static.service';
import { ServerController } from './server/server.controller';
import { ServerService } from './server/server.service';
import { PrometheusController } from './stats/prometheus.controller';
import { PrometheusService } from './stats/prometheus.service';
import { StatsResolver } from './stats/stats.resolver';
import { BigIntScalar } from './utils/bigint.scalar';

@Module({
  imports: [
    GlobalModule,
    ApplicationConfigModule,
    ...WoodstockBullModules,
    ConfigModule.forRoot(),
    GraphQLModule.forRoot({
      fieldResolverEnhancers: ['interceptors'],
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
  controllers: [
    QueueController,
    BackupController,
    HostController,
    ServerController,
    BackupsFilesController,
    PrometheusController,
  ],
  providers: [
    BackupController,
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
    ServerController,
    ServerService,
    ServeStaticService,
    StatsResolver,
    PrometheusService,
    {
      provide: 'BACKUP_QUEUE_PUB_SUB',
      useValue: new PubSub(),
    },
  ],
})
export class AppModule {}
