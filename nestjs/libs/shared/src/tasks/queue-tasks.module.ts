import { Module } from '@nestjs/common';
import { QueueTasksService } from './queue-tasks.service';

@Module({
  providers: [QueueTasksService],
  exports: [QueueTasksService],
})
export class QueueTasksModule {}
