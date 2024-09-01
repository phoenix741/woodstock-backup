import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import {
  ApplicationLogger,
  BackupsService,
  CacheConfigService,
  ConfigProviderModule,
  SharedModule,
} from '@woodstock/shared';
import { SchedulerConsumer } from './scheduler/scheduler.consumer.js';
import { SchedulerService } from './scheduler/scheduler.service.js';
import { StatsService } from './scheduler/stats.service.js';
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
    SchedulerConsumer,
    SchedulerService,
    StatsService,
    {
      provide: ApplicationLogger,
      useFactory: (backupsService) => new ApplicationLogger('stats', backupsService),
      inject: [BackupsService],
    },
  ],
})
export class AppModule {}
