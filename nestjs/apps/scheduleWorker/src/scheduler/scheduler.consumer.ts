import { InjectQueue, Processor, WorkerHost } from '@nestjs/bullmq';
import { Logger, NotFoundException } from '@nestjs/common';
import { BackupTask, HostsService, PoolService } from '@woodstock/shared';
import { Job, Queue } from 'bullmq';
import { count } from 'rxjs';

@Processor('schedule')
export class SchedulerConsumer extends WorkerHost {
  private logger = new Logger(SchedulerConsumer.name);

  constructor(
    @InjectQueue('queue') private hostsQueue: Queue<BackupTask>,
    private hostsService: HostsService,
    private poolService: PoolService,
  ) {
    super();
  }

  async process(job: Job<BackupTask>): Promise<void> {
    switch (job.name) {
      case 'wakeup':
        await this.wakeupJob(job);
        break;
      case 'nightly':
        await this.nightlyJob(job);
        break;
      default:
        throw new NotFoundException(`Unknown job name ${job.name}`);
    }
  }

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
