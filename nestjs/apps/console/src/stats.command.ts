import { InjectQueue, OnQueueEvent, QueueEventsHost, QueueEventsListener } from '@nestjs/bullmq';
import { JobBackupData, QueueName } from '@woodstock/server';
import { Queue } from 'bullmq';
import { Command, Console } from 'nestjs-console';
import * as ora from 'ora';

@Console({
  command: 'stats',
})
@QueueEventsListener(QueueName.BACKUP_QUEUE)
export class StatsCommand extends QueueEventsHost {
  private spinner?: ora.Ora;
  private jobId?: string;

  constructor(
    @InjectQueue(QueueName.STATS_QUEUE) private statsQueue: Queue<void>,
    @InjectQueue(QueueName.BACKUP_QUEUE) private hostsQueue: Queue<JobBackupData>,
  ) {
    super();
  }

  @Command({
    command: 'calc',
    description: 'Calculate daily statistics',
  })
  async statistics(): Promise<void> {
    this.spinner = ora();
    this.spinner.start('[Stats] Start');

    await this.statsQueue.add('stats');
    this.spinner.succeed('[Stats] In progress');
  }

  @OnQueueEvent('progress')
  async handler({ jobId }: { jobId: string }): Promise<void> {
    const job = await this.hostsQueue.getJob(jobId);
    if (job && job.id === this.jobId && this.spinner) {
      this.spinner.text = `[Stats] ${job.data.host}/${job.data.number}: Progress ${Math.round(
        (job.progress as number) * 100,
      )}%`;
    }
  }
}
