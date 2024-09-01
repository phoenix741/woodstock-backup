import { Module, OnApplicationBootstrap } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import {
  ApplicationConfigService,
  ApplicationLogger,
  BackupsService,
  CacheConfigService,
  ConfigProviderModule,
  initializeLog,
  SharedModule,
} from '@woodstock/shared';
import { RefcntConsumer } from './refcnt.consumer.js';
import { CacheModule } from '@nestjs/cache-manager';
import { IORedisOptions } from '@nestjs/microservices/external/redis.interface.js';

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
  providers: [
    RefcntConsumer,
    {
      provide: ApplicationLogger,
      useFactory: (backupsService) => new ApplicationLogger('refcnt', backupsService),
      inject: [BackupsService],
    },
  ],
})
export class AppModule implements OnApplicationBootstrap {
  constructor(private readonly config: ApplicationConfigService) {}

  async onApplicationBootstrap() {
    await initializeLog(this.config.context);
  }
}
