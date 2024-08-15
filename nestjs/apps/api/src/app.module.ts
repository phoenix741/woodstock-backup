import { ApolloDriver, ApolloDriverConfig } from '@nestjs/apollo';
import { Module, OnApplicationBootstrap } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { GraphQLModule } from '@nestjs/graphql';
import { ServeStaticModule } from '@nestjs/serve-static';
import {
  ApplicationConfigService,
  CertificateService,
  ConfigProviderModule,
  SharedModule,
  initializeLog,
} from '@woodstock/shared';
import { PubSub } from 'graphql-subscriptions';
import { BackupsFilesController } from './backups/backups-files.controller.js';
import { BackupsFilesService } from './backups/backups-files.service.js';
import { BackupController } from './backups/backups.controller.js';
import { BackupsResolver } from './backups/backups.resolver.js';
import { HostController } from './hosts/hosts.controller.js';
import { HostsResolver } from './hosts/hosts.resolver.js';
import { PoolResolver } from './pool/pool.resolver.js';
import { QueueController } from './queue/queue.controller.js';
import { QueueResolver } from './queue/queue.resolver.js';
import { QueueService } from './queue/queue.service.js';
import { QueueUtils } from './queue/queue.utils.js';
import { RefcntQueueService } from './queue/refcnt-queue.service.js';
import { ServeStaticService } from './server/serve-static.service.js';
import { ServerController } from './server/server.controller.js';
import { ServerService } from './server/server.service.js';
import { PrometheusController } from './stats/prometheus.controller.js';
import { PrometheusService } from './stats/prometheus.service.js';
import { StatsResolver } from './stats/stats.resolver.js';
import { BigIntScalar } from './utils/bigint.scalar.js';
import { generateRsaKey } from '@woodstock/shared-rs';
import { ServerResolver } from './server/server.resolver.js';
import { CacheModule } from '@nestjs/cache-manager';
import { IORedisOptions } from '@nestjs/microservices/external/redis.interface.js';
import { CacheConfigService } from '@woodstock/shared';

@Module({
  imports: [
    ConfigModule.forRoot({ isGlobal: true }),
    CacheModule.registerAsync<IORedisOptions>({
      isGlobal: true,
      useClass: CacheConfigService,
      imports: [ConfigProviderModule],
    }),
    ConfigProviderModule,
    SharedModule,
    GraphQLModule.forRoot<ApolloDriverConfig>({
      driver: ApolloDriver,
      fieldResolverEnhancers: ['interceptors'],
      installSubscriptionHandlers: true,
      autoSchemaFile: true,
      buildSchemaOptions: {
        dateScalarMode: 'isoDate',
      },
    }),
    ServeStaticModule.forRootAsync({
      useClass: ServeStaticService,
      imports: [ConfigProviderModule],
    }),
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
    CacheConfigService,
    BackupController,
    BackupsFilesController,
    BackupsFilesService,
    BackupsResolver,
    BigIntScalar,
    HostController,
    HostsResolver,
    QueueController,
    QueueResolver,
    QueueService,
    RefcntQueueService,
    QueueUtils,
    ServerController,
    ServerService,
    ServeStaticService,
    ServerResolver,
    StatsResolver,
    PrometheusService,
    PoolResolver,
    {
      provide: 'BACKUP_QUEUE_PUB_SUB',
      useValue: new PubSub(),
    },
  ],
})
export class AppModule implements OnApplicationBootstrap {
  constructor(
    private readonly serverService: ServerService,
    private readonly config: ApplicationConfigService,
    private readonly certificateService: CertificateService,
  ) {}

  async onApplicationBootstrap() {
    await initializeLog(this.config.context);

    await this.serverService.check();
    await this.certificateService.generateCertificate();
    generateRsaKey(this.config.context);
  }
}
