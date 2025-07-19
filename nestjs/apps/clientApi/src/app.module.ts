import { CacheModule } from '@nestjs/cache-manager';
import { Module, OnApplicationBootstrap } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { IORedisOptions } from '@nestjs/microservices/external/redis.interface.js';
import {
  ApplicationLogger,
  CacheConfigService,
  CertificateService,
  ConfigProviderModule,
  SharedModule,
  initializeLog,
} from '@woodstock/shared';
import { ClientCertificateStrategy } from './auth/client-strategy.service.js';
import { HostController } from './hosts/hosts.controller.js';
import { PubSub } from 'graphql-subscriptions';

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
  ],
  controllers: [HostController],
  providers: [
    CacheConfigService,
    HostController,
    ClientCertificateStrategy,
    {
      provide: 'BACKUP_QUEUE_PUB_SUB',
      useValue: new PubSub(),
    },
    {
      provide: ApplicationLogger,
      useFactory: () => new ApplicationLogger('main'),
    },
  ],
})
export class AppModule implements OnApplicationBootstrap {
  constructor(private readonly certificateService: CertificateService) {}

  async onApplicationBootstrap() {
    await initializeLog();

    await this.certificateService.generateHttpsCertificate();
  }
}
