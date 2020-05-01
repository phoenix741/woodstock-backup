import { InjectQueue, Process, Processor } from '@nestjs/bull';
import { Logger } from '@nestjs/common';
import { Job, Queue } from 'bull';

import { BackupTask } from '../tasks/tasks.dto';
import { HostsService } from 'src/hosts/hosts.service';

@Processor('schedule')
export class SchedulerConsumer {
  private logger = new Logger(SchedulerConsumer.name);

  constructor(@InjectQueue('queue') private hostsQueue: Queue<BackupTask>, private hostsService: HostsService) {}

  @Process()
  async schedule(job: Job<object>) {
    this.logger.log(`Scheduler wakeup at ${new Date().toISOString()} - JOB ID = ${job.id}`);
    for (const host of await this.hostsService.getHosts()) {
      const hasBackup = (await this.hostsQueue.getJobs(['active', 'delayed', 'waiting'])).find(
        b => b.data.host === host,
      );

      if (!hasBackup) {
        await this.hostsQueue.add('schedule_host', { host }, { removeOnComplete: true });
      }
    }
  }
}
