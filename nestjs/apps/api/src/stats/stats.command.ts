import { InjectQueue, OnGlobalQueueProgress, Processor } from '@nestjs/bull';
import { BackupTask } from '@woodstock/shared';
import { JobId, Queue } from 'bull';
import { Command as Cmd } from 'commander';
import { Command, Console, createSpinner } from 'nestjs-console';
import * as ora from 'ora';

@Console({
  command: 'stats',
})
@Processor('queue')
export class StatsCommand {
  private spinner?: ora.Ora;
  private jobId?: JobId;

  constructor(
    @InjectQueue('stats') private statsQueue: Queue<BackupTask>,
    @InjectQueue('queue') private hostsQueue: Queue<BackupTask>,
    @InjectQueue('schedule') private scheduleQueue: Queue<unknown>,
  ) {}

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

    let job = await this.statsQueue.add({ host, number }, { removeOnComplete: true });
    this.jobId = job.id;
    await job.finished();
    job = (await this.statsQueue.getJob(job.id)) || job;

    this.spinner.succeed(`[Stats] ${host}/${job.data.number || 'NA'}: Progress 100%`);
  }

  @Command({
    command: 'calc',
    description: 'Calculate daily statistics',
  })
  async statistics(): Promise<void> {
    this.spinner = createSpinner();
    this.spinner.start('[Stats] Start');

    await this.scheduleQueue.add('nightly', {}, { removeOnComplete: true });
    this.spinner.succeed('[Stats] In progress');
  }

  @OnGlobalQueueProgress()
  async handler(jobId: number, progress: number): Promise<void> {
    const job = await this.hostsQueue.getJob(jobId);
    if (job && job.id === this.jobId && this.spinner) {
      this.spinner.text = `[Stats] ${job.data.host}/${job.data.number}: Progress ${Math.round(progress * 100)}%`;
    }
  }
}
