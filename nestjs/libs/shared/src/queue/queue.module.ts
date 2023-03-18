import { BullModule } from '@nestjs/bullmq';
import { Module } from '@nestjs/common';
import { ApplicationConfigModule } from '../config';
import { BullConfigService } from './bull-config.factory';

export enum QueueName {
  BACKUP_QUEUE = 'backup',
  REFCNT_QUEUE = 'refcnt',
  SCHEDULE_QUEUE = 'schedule',
  STATS_QUEUE = 'stats',
}

const QUEUES = [QueueName.BACKUP_QUEUE, QueueName.REFCNT_QUEUE, QueueName.SCHEDULE_QUEUE, QueueName.STATS_QUEUE];

const RegisteredQueue = BullModule.registerQueue(...QUEUES.map((q) => ({ name: q })));

@Module({
  imports: [
    BullModule.forRootAsync({
      imports: [ApplicationConfigModule],
      useClass: BullConfigService,
    }),
    RegisteredQueue,
  ],
  // exports: [...QUEUES.map((q) => getQueueToken(q))],
  exports: [RegisteredQueue],
})
export class QueueModule {}
