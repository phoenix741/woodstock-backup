import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { CoreModule } from '@woodstock/core';
import { ServerModule } from '@woodstock/server';
import { SharedModule } from '@woodstock/shared';
import { GlobalModule } from './global.module.js';
import { SchedulerConsumer } from './scheduler/scheduler.consumer.js';
import { SchedulerService } from './scheduler/scheduler.service.js';

@Module({
  imports: [ConfigModule.forRoot(), GlobalModule, CoreModule, SharedModule, ServerModule],
  providers: [SchedulerConsumer, SchedulerService],
})
export class AppModule {}
