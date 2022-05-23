import { Module } from '@nestjs/common';
import { ApplicationConfigModule, SharedModule, WoodstockBullModules } from '@woodstock/shared';
import { SchedulerConsumer } from './scheduler/scheduler.consumer';
import { SchedulerService } from './scheduler/scheduler.service';

@Module({
  imports: [ApplicationConfigModule, ...WoodstockBullModules, SharedModule],
  providers: [SchedulerConsumer, SchedulerService],
})
export class AppModule {}
