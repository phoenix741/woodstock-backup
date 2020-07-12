import { InjectQueue, Process, Processor } from '@nestjs/bull';
import { Logger } from '@nestjs/common';
import { Job, Queue } from 'bull';
import { HostsService } from 'src/hosts/hosts.service';

import { StatsService } from '../stats/stats.service';
import { BackupTask } from '../tasks/tasks.dto';

@Processor('schedule')
export class SchedulerConsumer {
  private logger = new Logger(SchedulerConsumer.name);

  constructor(
    @InjectQueue('queue') private hostsQueue: Queue<BackupTask>,
    private hostsService: HostsService,
    private statsService: StatsService,
  ) {}

  @Process('wakeup')
  async wakeupJob(job: Job<{}>) {
    this.logger.log(`Wakeup scheduler wakeup at ${new Date().toISOString()} - JOB ID = ${job.id}`);
    for (const host of await this.hostsService.getHosts()) {
      const hasBackup = (await this.hostsQueue.getJobs(['active', 'delayed', 'waiting'])).find(
        b => b.data.host === host,
      );

      if (!hasBackup) {
        await this.hostsQueue.add('schedule_host', { host }, { removeOnComplete: true });
      }
    }
  }

  @Process('nightly')
  async nightlyJob(job: Job<{}>) {
    this.logger.log(`Nightly scheduler wakeup at ${new Date().toISOString()} - JOB ID = ${job.id}`);
    this.statsService.refreshStatistics();
  }
}
