import { Module } from '@nestjs/common';
import { ApplicationConfigModule, BackofficeSharedModule, WoodstockBullModules } from '@woodstock/backoffice-shared';
import { SharedModule } from '@woodstock/shared';
import { SchedulerConsumer } from './scheduler/scheduler.consumer';
import { SchedulerService } from './scheduler/scheduler.service';

@Module({
  imports: [ApplicationConfigModule, BackofficeSharedModule, ...WoodstockBullModules, SharedModule],
  providers: [SchedulerConsumer, SchedulerService],
})
export class AppModule {}
