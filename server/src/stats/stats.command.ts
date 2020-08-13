import { InjectQueue, OnQueueProgress, Processor } from '@nestjs/bull';
import { Job, JobId, Queue } from 'bull';
import { Command, Console, createSpinner } from 'nestjs-console';
import { Command as Cmd } from 'commander';
import * as ora from 'ora';

import { BackupTask } from '../tasks/tasks.dto';

@Console({
  name: 'stats',
})
@Processor('queue')
export class StatsCommand {
  private spinner?: ora.Ora;
  private jobId?: JobId;

  constructor(
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
      },
    ],
  })
  async create(host: string, cmd: Cmd): Promise<void> {
    const { number } = cmd.opts();
    this.spinner = createSpinner();
    this.spinner.start(`[Stats] ${host}/${number || 'NA'}: Progress 0%`);

    const job = await this.hostsQueue.add('stats', { host, number: parseInt(number) }, { removeOnComplete: true });
    this.jobId = job.id;
    await job.finished();

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

  @OnQueueProgress()
  handler(job: Job<BackupTask>, progress: number): void {
    if (job.id === this.jobId && this.spinner) {
      this.spinner.text = `[Stats] ${job.data.host}/${job.data.number}: Progress ${Math.round(progress * 100)}%`;
    }
  }
}
