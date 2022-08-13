import { Module } from '@nestjs/common';
import { ApplicationConfigModule, SharedModule, WoodstockBullModules } from '@woodstock/shared';
import { GlobalModule } from './global.module.js';
import { SchedulerConsumer } from './scheduler/scheduler.consumer.js';
import { SchedulerService } from './scheduler/scheduler.service.js';

@Module({
  imports: [GlobalModule, ApplicationConfigModule, ...WoodstockBullModules, SharedModule],
  providers: [SchedulerConsumer, SchedulerService],
})
export class AppModule {}
