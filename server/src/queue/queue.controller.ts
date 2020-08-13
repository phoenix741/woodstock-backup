import { InjectQueue } from '@nestjs/bull';
import { Controller, Get } from '@nestjs/common';
import { BackupTask } from '../tasks/tasks.dto';
import { Queue } from 'bull';
import { ApiOkResponse } from '@nestjs/swagger';

@Controller('queue')
export class QueueController {
  constructor(@InjectQueue('queue') private queue: Queue<BackupTask>) {}

  @Get()
  @ApiOkResponse({
    description: 'Return the list of task',
    type: [BackupTask],
  })
  async list(): Promise<BackupTask[]> {
    return (await this.queue.getJobs(['waiting', 'active', 'failed', 'delayed'])).map((job) => job.data);
  }
}
