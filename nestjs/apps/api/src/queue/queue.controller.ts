import { InjectQueue } from '@nestjs/bullmq';
import { Controller, Get } from '@nestjs/common';
import { ApiOkResponse } from '@nestjs/swagger';
import { BackupTask, JobBackupData, QueueName } from '@woodstock/server';
import { Queue } from 'bullmq';

@Controller('queue')
export class QueueController {
  constructor(@InjectQueue(QueueName.BACKUP_QUEUE) private queue: Queue<JobBackupData>) {}

  @Get()
  @ApiOkResponse({
    description: 'Return the list of task',
    type: [BackupTask],
  })
  async list(): Promise<JobBackupData[]> {
    return (await this.queue.getJobs(['waiting', 'active', 'failed', 'delayed'])).map((job) => job.data);
  }
}
