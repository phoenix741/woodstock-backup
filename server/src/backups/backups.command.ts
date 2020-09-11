import { InjectQueue, OnGlobalQueueProgress, Processor } from '@nestjs/bull';
import { JobId, Queue } from 'bull';
import { Command, Console, createSpinner } from 'nestjs-console';
import * as ora from 'ora';

import { HostsService } from '../hosts/hosts.service';
import { BackupTask } from '../tasks/tasks.dto';

@Console({
  name: 'backups',
})
@Processor('queue')
export class BackupsCommand {
  private spinner?: ora.Ora;
  private jobId?: JobId;

  constructor(@InjectQueue('queue') private hostsQueue: Queue<BackupTask>, private hostsService: HostsService) {}

  @Command({
    command: 'import <hostname> <date> <path>',
    description: 'Import backup from filesystem',
  })
  async import(host: string, date: number, pathPrefix: string): Promise<void> {
    this.spinner = createSpinner();
    this.spinner.start(`[Backups/Import] ${host}/NA: Progress 0%`);

    const config = await this.hostsService.getHostConfiguration(host);
    config.isLocal = true;
    config.operations = config.operations || {};
    config.operations.finalizeTasks = [];
    config.operations.tasks = config.operations.tasks
      ?.filter((operation) => operation.name !== 'ExecuteCommand')
      .map((operation) => {
        switch (operation.name) {
          case 'RSyncBackup':
            return {
              ...operation,
              share: operation.share.map((s) => ({ ...s, pathPrefix })),
            };
          case 'RSyncdBackup':
            return {
              name: 'RSyncBackup',
              share: operation.share.map((s) => ({ ...s, pathPrefix })),
              includes: operation.includes,
              excludes: operation.excludes,
              timeout: operation.timeout,
            };
          default:
            return operation;
        }
      });

    let job = await this.hostsQueue.add(
      'backup',
      {
        config,
        host,
        originalStartDate: date,
      },
      { removeOnComplete: true },
    );
    this.jobId = job.id;
    await job.finished();
    job = (await this.hostsQueue.getJob(job.id)) || job;

    this.spinner.succeed(`[Backups/Import] ${host}/${job.data.number || 'NA'}: Progress 100%`);
  }

  @OnGlobalQueueProgress()
  async handler(jobId: number, progress: number): Promise<void> {
    const job = await this.hostsQueue.getJob(jobId);
    if (job && job.id === this.jobId && this.spinner) {
      this.spinner.text = `[Backups/Import] ${job.data.host}/${job.data.number}: Progress ${Math.round(
        progress / 100,
      )}%`;
    }
  }
}
