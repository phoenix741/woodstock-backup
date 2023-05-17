import { InjectQueue, Processor, WorkerHost } from '@nestjs/bullmq';
import { Logger, NotFoundException } from '@nestjs/common';
import { HostsService, JobBackupData, JobService, QueueName, RefcntJobData } from '@woodstock/server';
import { Job, Queue } from 'bullmq';

@Processor('schedule')
export class SchedulerConsumer extends WorkerHost {
  private logger = new Logger(SchedulerConsumer.name);

  constructor(
    @InjectQueue(QueueName.BACKUP_QUEUE) private hostsQueue: Queue<JobBackupData>,
    @InjectQueue(QueueName.REFCNT_QUEUE) private refcntQueue: Queue<RefcntJobData>,
    @InjectQueue(QueueName.STATS_QUEUE) private statsQueue: Queue<void>,
    private hostsService: HostsService,
    private jobService: JobService,
  ) {
    super();
  }

  async process(job: Job<JobBackupData>): Promise<void> {
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
      const hasBackup = await this.jobService.shouldBackupHost(host);
      if (!hasBackup) {
        await this.hostsQueue.add('backup', { host });
      }
    }
  }

  async nightlyJob(job: Job<unknown>): Promise<void> {
    this.logger.log(`Nightly scheduler wakeup at ${new Date().toISOString()} - JOB ID = ${job.id}`);
    this.statsQueue.add('stats');
    this.refcntQueue.add('unused', {});
  }
}
