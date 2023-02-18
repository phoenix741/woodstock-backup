import { InjectQueue, Processor, WorkerHost } from '@nestjs/bullmq';
import { Logger, NotFoundException } from '@nestjs/common';
import { BackupTask, HostsService, JobService, RefcntJobData } from '@woodstock/shared';
import { JobBackupData } from '@woodstock/shared/backuping/backuping.model';
import { Job, Queue } from 'bullmq';

@Processor('schedule')
export class SchedulerConsumer extends WorkerHost {
  private logger = new Logger(SchedulerConsumer.name);

  constructor(
    @InjectQueue('queue') private hostsQueue: Queue<JobBackupData>,
    @InjectQueue('refcnt') private refcntQueue: Queue<RefcntJobData>,
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
    this.refcntQueue.add('unused', {});
  }
}
