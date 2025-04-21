import { BullModule } from '@nestjs/bullmq';

export enum QueueName {
  BACKUP_QUEUE = 'backup',
  SCHEDULE_QUEUE = 'schedule',
}

const QUEUES = [QueueName.BACKUP_QUEUE, QueueName.SCHEDULE_QUEUE];

export const RegisteredQueue = BullModule.registerQueue(...QUEUES.map((q) => ({ name: q })));
