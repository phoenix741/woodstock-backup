import { InjectQueue, OnGlobalQueueProgress, Processor } from '@nestjs/bull';
import { JobId, Queue } from 'bull';
import { promises as fs } from 'fs';
import { Command, Console, createSpinner } from 'nestjs-console';
import { Command as Cmd } from 'commander';
import * as ora from 'ora';
import { join } from 'path';

import { HostsService } from '../hosts/hosts.service';
import { BackupTask } from '../tasks/tasks.dto';

interface BackupPCSlot {
  date: number;
  path: string;
}

const DATEISO8601 = /\d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}/;

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
  async import(host: string, date: string | number, pathPrefix: string, prefixText?: string): Promise<void> {
    this.spinner = createSpinner({ prefixText });
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
        originalStartDate: typeof date === 'string' ? parseInt(date) : date,
      },
      { removeOnComplete: true },
    );
    try {
      this.jobId = job.id;
      await job.finished();
      job = (await this.hostsQueue.getJob(job.id)) || job;

      if (await job.isFailed()) {
        throw new Error(job.failedReason);
      }

      this.spinner.succeed(`[Backups/Import] ${host}/${job.data.number || 'NA'}: Progress 100%`);
    } catch (err) {
      job = (await this.hostsQueue.getJob(job.id)) || job;
      this.spinner.fail(`[Backups/Import] ${host}/${job.data.number || 'NA'}: ${err.message}`);
    }
  }

  @Command({
    command: 'backuppc <path>',
    description:
      'Import backup from backuppc fuse directory. The fuse driver will present file in a flat view with a link for each backup that target the real backup. (See https://sourceforge.net/p/backuppc/mailman/message/35899426/)',
    options: [
      {
        flags: '-h, --host <host>',
        required: false,
        description: 'Host to select for the backup (to limit the backup)',
      },
      {
        flags: '-s, --start-date <date>',
        required: false,
        description: 'The minimum date to take to import of backup',
        fn: parseInt,
      },
      {
        flags: '-e, --end-date <date>',
        required: false,
        description: 'The maximum date to take to import of backup',
        fn: parseInt,
      },
    ],
  })
  async importFromBackuppc(path: string, cmd: Cmd): Promise<void> {
    const { host, startDate, endDate } = cmd.opts();
    const hosts = await this.hostsService.getHosts();
    const originalBackup: Record<string, BackupPCSlot[]> = {};
    const files = await fs.readdir(path);
    for (const file of files) {
      if (!hosts.includes(file)) continue;
      if (!!host && file !== host) continue;

      const backups: BackupPCSlot[] = [];

      const backupDirEnts = await fs.readdir(join(path, file), { withFileTypes: true });
      for (const dirEntry of backupDirEnts) {
        if (DATEISO8601.test(dirEntry.name)) {
          const date = Date.parse(dirEntry.name);
          if (startDate && date < startDate) continue;
          if (endDate && date > endDate) continue;

          let name = dirEntry.name;
          if (dirEntry.isSymbolicLink()) {
            name = await fs.readlink(join(path, file, dirEntry.name));
          }
          backups.push({ date, path: join(path, file, name) });
        }
      }

      if (backups.length) {
        originalBackup[file] = backups;
      }
    }

    for (const host in originalBackup) {
      const backups = originalBackup[host];
      let importedBackup = 0;

      for (const backup of backups) {
        const globalProgress = `${host}(${importedBackup}/${backups.length})`;

        await this.import(host, backup.date, backup.path, globalProgress);

        importedBackup++;
      }
    }
  }

  @OnGlobalQueueProgress()
  async handler(jobId: number, progress: number): Promise<void> {
    const job = await this.hostsQueue.getJob(jobId);
    if (job && job.id === this.jobId && this.spinner) {
      this.spinner.text = `[Backups/Import] ${job.data.host}/${job.data.number}: Progress ${Math.round(progress)}%`;
    }
  }
}
