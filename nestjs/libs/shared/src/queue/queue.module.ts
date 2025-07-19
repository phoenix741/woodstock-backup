import { BullModule } from '@nestjs/bullmq';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

export enum QueueName {
  BACKUP_QUEUE = 'backup',
  SCHEDULE_QUEUE = 'schedule',
}

export const MAX_BACKUP_TASK = parseInt(process.env.MAX_BACKUP_TASK ?? '1', 10);

export const RegisteredQueue = BullModule.registerQueue(
  {
    name: QueueName.BACKUP_QUEUE,
    processors: [
      {
        concurrency: MAX_BACKUP_TASK,
        path: pathToFileURL(join(__dirname, '..', 'backupWorker', 'main.js')),
      },
    ],
  },
  {
    name: QueueName.SCHEDULE_QUEUE,
  },
);
