import { InjectQueue, Process, Processor } from '@nestjs/bull';
import { Logger } from '@nestjs/common';
import { BackupTask, HostsService, PoolService } from '@woodstock/shared';
import { Job, Queue } from 'bull';
import { count, reduce } from 'rxjs';

@Processor('schedule')
export class SchedulerConsumer {
  private logger = new Logger(SchedulerConsumer.name);

  constructor(
    @InjectQueue('queue') private hostsQueue: Queue<BackupTask>,
    private hostsService: HostsService,
    private poolService: PoolService,
  ) {}

  @Process('wakeup')
  async wakeupJob(job: Job<unknown>): Promise<void> {
    this.logger.log(`Wakeup scheduler wakeup at ${new Date().toISOString()} - JOB ID = ${job.id}`);
    for (const host of await this.hostsService.getHosts()) {
      const hasBackup = (await this.hostsQueue.getJobs(['active', 'delayed', 'waiting'])).find(
        (b) => b.data.host === host,
      );

      if (!hasBackup) {
        await this.hostsQueue.add('schedule_host', { host }, { removeOnComplete: true });
      }
    }
  }

  @Process('nightly')
  async nightlyJob(job: Job<unknown>): Promise<void> {
    this.logger.log(`Nightly scheduler wakeup at ${new Date().toISOString()} - JOB ID = ${job.id}`);
    this.poolService
      .removeUnusedFiles()
      .pipe(count())
      .subscribe({
        next: (count) => {
          this.logger.log(`Removed ${count} unused chunks`);
        },
      });
  }
}
