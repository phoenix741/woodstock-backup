import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { BackupingModule, CommandsModule, CoreModule, QueueModule } from '@woodstock/shared';
import { GlobalModule } from './global.module.js';
import { SchedulerConsumer } from './scheduler/scheduler.consumer.js';
import { SchedulerService } from './scheduler/scheduler.service.js';

@Module({
  imports: [ConfigModule.forRoot(), GlobalModule, CoreModule, CommandsModule, QueueModule, BackupingModule],
  providers: [SchedulerConsumer, SchedulerService],
})
export class AppModule {}
