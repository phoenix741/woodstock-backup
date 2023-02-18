import { InjectQueue, OnQueueEvent, QueueEventsHost, QueueEventsListener } from '@nestjs/bullmq';
import { JobBackupData } from '@woodstock/shared/backuping/backuping.model';
import { Queue } from 'bullmq';
import { Command as Cmd } from 'commander';
import { Command, Console, createSpinner } from 'nestjs-console';

@Console({
  command: 'stats',
})
@QueueEventsListener('queue')
export class StatsCommand extends QueueEventsHost {
  private spinner?: ReturnType<typeof createSpinner>;
  private jobId?: string;

  constructor(
    @InjectQueue('stats') private statsQueue: Queue<JobBackupData>,
    @InjectQueue('queue') private hostsQueue: Queue<JobBackupData>,
    @InjectQueue('schedule') private scheduleQueue: Queue<unknown>,
  ) {
    super();
  }

  @Command({
    command: 'host <hostname>',
    description: 'Calculate stats for a host',
    options: [
      {
        flags: '-n, --number <number>',
        description: 'Backup number',
        fn: parseInt,
      },
    ],
  })
  async create(host: string, cmd: Cmd): Promise<void> {
    const { number } = cmd.opts();
    this.spinner = createSpinner();
    this.spinner.start(`[Stats] ${host}/${number || 'NA'}: Progress 0%`);

    let job = await this.statsQueue.add('stats', { host, number });
    this.jobId = job.id;
    if (!this.jobId) {
      throw new Error('Job ID is not defined');
    }

    await job.waitUntilFinished(this.queueEvents);
    job = (await this.statsQueue.getJob(this.jobId)) || job;

    this.spinner.succeed(`[Stats] ${host}/${job.data.number || 'NA'}: Progress 100%`);
  }

  @Command({
    command: 'calc',
    description: 'Calculate daily statistics',
  })
  async statistics(): Promise<void> {
    this.spinner = createSpinner();
    this.spinner.start('[Stats] Start');

    await this.scheduleQueue.add('nightly', {});
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
