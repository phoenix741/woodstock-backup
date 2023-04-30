import { InjectQueue, OnQueueEvent, QueueEventsHost, QueueEventsListener } from '@nestjs/bullmq';
import { QueueName } from '@woodstock/shared';
import { JobBackupData } from '@woodstock/shared/backuping/backuping.model';
import { Queue } from 'bullmq';
import { Command, Console, createSpinner } from 'nestjs-console';

@Console({
  command: 'stats',
})
@QueueEventsListener(QueueName.BACKUP_QUEUE)
export class StatsCommand extends QueueEventsHost {
  private spinner?: ReturnType<typeof createSpinner>;
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
    this.spinner = createSpinner();
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
