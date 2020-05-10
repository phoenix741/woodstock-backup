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

  constructor(@InjectQueue('queue') private hostsQueue: Queue<BackupTask>) {}

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
  async create(host: string, cmd: Cmd) {
    const { number } = cmd.opts();
    this.spinner = createSpinner();
    this.spinner.start(`${host}/${number || 'NA'}: Progress 0%`);

    const job = await this.hostsQueue.add('stats', { host, number: parseInt(number) }, { removeOnComplete: true });
    this.jobId = job.id;
    await job.finished();

    this.spinner.succeed(`${host}/${job.data.number || 'NA'}: Progress 100%`);
  }

  @OnQueueProgress()
  handler(job: Job<BackupTask>, progress: number) {
    if (job.id === this.jobId && this.spinner) {
      this.spinner.text = `${job.data.host}/${job.data.number}: Progress ${Math.round(progress * 100)}%`;
    }
  }
}
