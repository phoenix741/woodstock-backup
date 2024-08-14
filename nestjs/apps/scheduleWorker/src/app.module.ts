import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { ConfigProviderModule, SharedModule } from '@woodstock/shared';
import { SchedulerConsumer } from './scheduler/scheduler.consumer.js';
import { SchedulerService } from './scheduler/scheduler.service.js';
import { StatsService } from './scheduler/stats.service.js';

@Module({
  imports: [ConfigModule.forRoot({ isGlobal: true }), ConfigProviderModule, SharedModule],
  providers: [SchedulerConsumer, SchedulerService, StatsService],
})
export class AppModule {}
