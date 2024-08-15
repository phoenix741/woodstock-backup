import { BullModule } from '@nestjs/bullmq';

export enum QueueName {
  BACKUP_QUEUE = 'backup',
  REFCNT_QUEUE = 'refcnt',
  SCHEDULE_QUEUE = 'schedule',
}

const QUEUES = [QueueName.BACKUP_QUEUE, QueueName.REFCNT_QUEUE, QueueName.SCHEDULE_QUEUE];

export const RegisteredQueue = BullModule.registerQueue(...QUEUES.map((q) => ({ name: q })));
