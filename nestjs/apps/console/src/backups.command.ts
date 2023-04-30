import { InjectQueue, QueueEventsHost } from '@nestjs/bullmq';
import { Inject } from '@nestjs/common';
import { HostsService, QueueName } from '@woodstock/shared';
import { JobBackupData } from '@woodstock/shared/backuping/backuping.model';
import { QueueGroupTasks, QueueSubTask, QueueTasks, QueueTaskState } from '@woodstock/shared/tasks';
import { Job, Queue } from 'bullmq';
import { promises as fs } from 'fs';
import { Command, Console, createSpinner } from 'nestjs-console';
import * as ora from 'ora';
import { join } from 'path';
import { BackupQueueStatus, QueueStatusInterface } from './queue-status.service';

interface BackupPCSlot {
  host: string;
  date: number;
  path: string;
}

const DATEISO8601 = /\d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}/;

function getRunningTask(task: QueueTasks): string {
  const runningSubtask = task.subtasks.find((subtask) => subtask.state === QueueTaskState.RUNNING);

  if (runningSubtask instanceof QueueSubTask) {
    switch (runningSubtask.taskName) {
      case 'init':
        return 'Initialisation';
      case 'share':
        return `Backup of share ${runningSubtask.localContext.sharePath?.toString() ?? ''}`;
      case 'pre-command':
        return 'Execution of distant command before backup';
      case 'post-command':
        return 'Execution of distant command after backup';
      case 'end':
        return 'Finalisation';

      case 'connection':
        return `Connection to host`;
      case 'init-directory':
        return 'Creation of backup directory';
      case 'refreshcache':
        return 'Refresh of client file list';
      case 'filelist':
        return 'Get the file list from the client';
      case 'chunks':
        return 'Get the chunks from the client';
      case 'compact':
        return 'Compact the manifest';
      case 'close-connection':
        return 'Close the client connection';
      case 'refcnt-host':
        return 'Count number of reference of the host';
      case 'refcnt-pool':
        return 'Refresh number of reference of the pool';
    }
    return runningSubtask.taskName;
  } else if (runningSubtask instanceof QueueGroupTasks) {
    return getRunningTask(runningSubtask);
  }
  return '';
}

@Console({
  command: 'backups',
})
export class BackupsCommand extends QueueEventsHost {
  constructor(
    @InjectQueue(QueueName.BACKUP_QUEUE) private hostsQueue: Queue<JobBackupData>,
    @Inject(BackupQueueStatus) private queueStatus: QueueStatusInterface<JobBackupData>,
    private hostsService: HostsService,
  ) {
    super();
  }

  async #waitingForJob(spinner: ora.Ora, job: Job<JobBackupData>) {
    await new Promise<void>((resolve, reject) => {
      let status = '';
      this.queueStatus.waitingJob(job).subscribe({
        next: (task) => {
          const progress =
            task.progression.progressMax > 0n
              ? (task.progression.progressCurrent * 100n) / task.progression.progressMax
              : 0n;

          const text = `[Backups/Import] - Progress ${Number(progress)}% - ${getRunningTask(task)}`;
          status = '';

          if (task.progression.fileCount > 0) {
            status += ` - ${task.progression.fileCount.toLocaleString()} files`;
          }
          if (task.progression.errorCount > 0) {
            status += ` - ${task.progression.errorCount.toLocaleString()} errors`;
          }
          if (task.progression.fileSize > 0) {
            status += ` - ${task.progression.fileSize.toLocaleString()}  bytes`;
          }
          if (task.progression.compressedFileSize > 0) {
            status += ` - ${task.progression.compressedFileSize.toLocaleString()} compressed bytes`;
          }

          spinner.text = text + status;
        },
        error: (err) => {
          spinner.fail(`[Backups/Import]: ${(err as Error).message} ${status}`);
          console.log(err);
          reject(err);
        },
        complete: async () => {
          spinner.succeed(`[Backups/Import]: Progress 100% - ${status}`);
          if (job.id) {
            console.log(
              (await this.hostsQueue.getJobLogs(job.id)).logs.filter((line) => line.indexOf('[DEBUG]') < 0).join('\n'),
            );
          }
          resolve();
        },
      });
    });
  }

  @Command({
    command: 'import <hostname> <date> <path>',
    description: 'Import backup from filesystem',
  })
  async import(host: string, date: string | number, pathPrefix: string, prefixText?: string): Promise<void> {
    console.log(`[Backups/Import] ${host}/NA: Importing backup from ${pathPrefix} for ${host} at ${date}`);
    const spinner = createSpinner({ prefixText });
    spinner.start(`[Backups/Import] ${host}/NA: Progress 0%`);

    const config = await this.hostsService.getHostConfiguration(host);
    config.isLocal = true;
    config.operations = config.operations || {};
    delete config.operations.preCommands;
    delete config.operations.postCommands;

    const job = await this.hostsQueue.add('backup', {
      config,
      host,

      pathPrefix,
      originalStartDate: typeof date === 'string' ? new Date(date).getTime() : date,
      force: true,
    });

    await this.#waitingForJob(spinner, job);
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
  async importFromBackuppc(
    path: string,
    { host, startDate, endDate }: { host?: string; startDate?: number; endDate?: number } = {},
  ): Promise<void> {
    const hosts = await this.hostsService.getHosts();
    const originalBackup: BackupPCSlot[] = [];
    const files = await fs.readdir(path);

    for (const file of files) {
      if (!hosts.includes(file)) continue;
      if (!!host && file !== host) continue;

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
          originalBackup.push({ host: file, date, path: join(path, file, name) });
        }
      }
    }

    // Sort by date asc
    originalBackup.sort((a, b) => a.date - b.date);

    for (const backup of originalBackup) {
      let importedBackup = 0;

      const globalProgress = `${backup.host}(${importedBackup}/${originalBackup.length})`;

      await this.import(backup.host, backup.date, backup.path, globalProgress);

      importedBackup++;
    }
  }
}
