import { Module, OnApplicationBootstrap } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { GraphQLModule } from '@nestjs/graphql';
import { ApolloDriver, ApolloDriverConfig } from '@nestjs/apollo';
import { ServeStaticModule } from '@nestjs/serve-static';
import { ApplicationConfigModule, SharedModule, WoodstockBullModules } from '@woodstock/shared';
import { PubSub } from 'graphql-subscriptions';
import { BackupsFilesController } from './backups/backups-files.controller.js';
import { BackupsFilesService } from './backups/backups-files.service.js';
import { BackupController } from './backups/backups.controller.js';
import { BackupsResolver } from './backups/backups.resolver.js';
import { GlobalModule } from './global.module.js';
import { HostController } from './hosts/hosts.controller.js';
import { HostsResolver } from './hosts/hosts.resolver.js';
import { JobResolver } from './queue/job.resolver.js';
import { QueueController } from './queue/queue.controller.js';
import { QueueResolver } from './queue/queue.resolver.js';
import { QueueService } from './queue/queue.service.js';
import { ServeStaticService } from './server/serve-static.service.js';
import { ServerController } from './server/server.controller.js';
import { ServerService } from './server/server.service.js';
import { PrometheusController } from './stats/prometheus.controller.js';
import { PrometheusService } from './stats/prometheus.service.js';
import { StatsResolver } from './stats/stats.resolver.js';
import { BigIntScalar } from './utils/bigint.scalar.js';

@Module({
  imports: [
    GlobalModule,
    ApplicationConfigModule,
    ...WoodstockBullModules,
    ConfigModule.forRoot(),
    GraphQLModule.forRoot<ApolloDriverConfig>({
      driver: ApolloDriver,
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
export class AppModule implements OnApplicationBootstrap {
  constructor(private readonly serverService: ServerService) {}

  async onApplicationBootstrap() {
    await this.serverService.check();
  }
}
