import { InjectQueue } from '@nestjs/bullmq';
import { Controller, Get } from '@nestjs/common';
import { ApiOkResponse } from '@nestjs/swagger';
import { BackupTask } from '@woodstock/shared';
import { JobBackupData } from '@woodstock/shared/backuping/backuping.model';
import { Queue } from 'bullmq';

@Controller('queue')
export class QueueController {
  constructor(@InjectQueue('queue') private queue: Queue<JobBackupData>) {}

  @Get()
  @ApiOkResponse({
    description: 'Return the list of task',
    type: [BackupTask],
  })
  async list(): Promise<JobBackupData[]> {
    return (await this.queue.getJobs(['waiting', 'active', 'failed', 'delayed'])).map((job) => job.data);
  }
}
