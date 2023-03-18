import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { CommandsModule, CoreModule, MaintenanceModule, QueueModule } from '@woodstock/shared';
import { QueueTasksModule } from '@woodstock/shared/tasks/queue-tasks.module.js';
import { GlobalModule } from './global.module.js';
import { RefcntConsumer } from './refcnt.consumer.js';

@Module({
  imports: [
    ConfigModule.forRoot(),
    GlobalModule,
    CoreModule,
    CommandsModule,
    QueueModule,
    QueueTasksModule,
    MaintenanceModule,
  ],
  providers: [RefcntConsumer],
})
export class AppModule {}
