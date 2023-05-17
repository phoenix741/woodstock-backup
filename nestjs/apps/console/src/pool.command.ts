import { InjectQueue } from '@nestjs/bullmq';
import { Inject } from '@nestjs/common';
import { ApplicationConfigService } from '@woodstock/core';
import { FsckService, PoolService, QueueName, RefcntJobData } from '@woodstock/server';
import { QueueGroupTasks, QueueSubTask, QueueTasks, QueueTaskState } from '@woodstock/server/tasks';
import { Job, Queue } from 'bullmq';
import { Command, Console, createSpinner } from 'nestjs-console';
import * as ora from 'ora';
import { join } from 'path';
import { pipeline } from 'stream/promises';
import { QueueStatusInterface, RefcntQueueStatus } from './queue-status.service';

function getRunningTask(task: QueueTasks): string {
  const runningSubtask = task.subtasks.find((subtask) => subtask.state === QueueTaskState.RUNNING);

  if (runningSubtask instanceof QueueSubTask) {
    switch (runningSubtask.taskName) {
      case 'prepare':
        return 'Prepare verification';
      case 'refcnt_backup':
        return `Count number of reference of the backup ${runningSubtask.localContext.host} - ${runningSubtask.localContext.number}`;
      case 'refcnt_host':
        return `Count number of reference of the host ${runningSubtask.localContext.host}`;
      case 'refcnt_pool':
        return 'Count number of reference of the pool';
      case 'refcnt_unused':
        return 'Count number of reference of the unused';
      case 'VERIFY_CHECKSUM':
        return 'Verify checksum';
    }
    return runningSubtask.taskName;
  } else if (runningSubtask instanceof QueueGroupTasks) {
    return getRunningTask(runningSubtask);
  }
  return '';
}

@Console({
  command: 'pool',
})
export class PoolCommand {
  constructor(
    private configService: ApplicationConfigService,
    @InjectQueue(QueueName.REFCNT_QUEUE) private refcntQueue: Queue<RefcntJobData>,
    private refCntFsckService: FsckService,
    private poolService: PoolService,
    @Inject(RefcntQueueStatus) private queueStatus: QueueStatusInterface<RefcntJobData>,
  ) {}

  async #waitingForJob(spinner: ora.Ora, job: Job<RefcntJobData>) {
    await new Promise<void>((resolve, reject) => {
      let status = '';
      this.queueStatus.waitingJob(job).subscribe({
        next: (task) => {
          const progress =
            task.progression.progressMax > 0n
              ? (task.progression.progressCurrent * 100n) / task.progression.progressMax
              : 0n;

          const text = `[Pool] - Progress ${Number(progress)}% - ${getRunningTask(task)}`;
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
          spinner.fail(`[Pool]: ${(err as Error).message} ${status}`);
          console.log(err);
          reject(err);
        },
        complete: async () => {
          spinner.succeed(`[Pool]: Verification finished. ${status}`);
          if (job.id) {
            console.log(`See results in ${join(this.configService.jobPath, job.id)}`);
          }
          resolve();
        },
      });
    });
  }

  @Command({
    command: 'fsck',
    description: 'Verify reference count',
    options: [
      {
        flags: '-f, --fix',
        required: false,
        description: 'Fix the reference count',
      },
    ],
  })
  async fsck({ fix }: { fix?: boolean }) {
    const spinner = createSpinner();
    spinner.start(`[Pool]: Progress 0%`);

    const fsckJob = await this.refcntQueue.add('fsck', { fix: !!fix, refcnt: true, unused: true });
    await this.#waitingForJob(spinner, fsckJob);
  }

  @Command({
    command: 'verify-refcnt',
    description: 'Verify the reference count of all backups',
    options: [
      {
        flags: '-f, --fix',
        required: false,
        description: 'Fix the reference count',
      },
    ],
  })
  async verifyRefcnt({ fix }: { fix?: boolean }) {
    const spinner = createSpinner();
    spinner.start(`[Pool]: Progress 0%`);

    const fsckJob = await this.refcntQueue.add('fsck', { fix: !!fix, refcnt: true, unused: false });
    await this.#waitingForJob(spinner, fsckJob);
  }

  @Command({
    command: 'verify-unused',
    description: 'Verify all the unused chunks',
    options: [
      {
        flags: '-f, --fix',
        required: false,
        description: 'Fix the reference count',
      },
    ],
  })
  async verifyUnused({ fix }: { fix?: boolean }) {
    const spinner = createSpinner();
    spinner.start(`[Pool]: Progress 0%`);

    const fsckJob = await this.refcntQueue.add('fsck', { fix: !!fix, refcnt: false, unused: true });
    await this.#waitingForJob(spinner, fsckJob);
  }

  @Command({
    command: 'remove-unused',
    description: 'Remove unused chunks',
    options: [
      {
        flags: '--target <path>',
        required: false,
        description: 'Target path to move file instead of deleting',
      },
    ],
  })
  async removeUnused({ target }: { target?: string }) {
    const spinner = createSpinner();
    spinner.start(`[Pool]: Progress 0%`);

    const fsckJob = await this.refcntQueue.add('unused', { target });
    await this.#waitingForJob(spinner, fsckJob);
  }

  @Command({
    command: 'verify-chunk',
    description: 'Verify the integrity of all chunk',
  })
  async verifyChunks() {
    const spinner = createSpinner();
    spinner.start(`[Pool]: Progress 0%`);

    const fsckJob = await this.refcntQueue.add('verify_checksum', {});
    await this.#waitingForJob(spinner, fsckJob);
  }

  @Command({
    command: 'get-chunk <chunk>',
    description: 'Read a chunk from the pool and decompress it',
  })
  async getChunk(chunk: string) {
    const buffChunk = Buffer.from(chunk, 'hex');
    const readable = this.poolService.getChunk(buffChunk).read();
    await pipeline(readable, process.stdout);
  }

  @Command({
    command: 'check-compression',
    description: 'Check the compression of all chunk',
    options: [
      {
        flags: '--all',
        required: false,
        description: 'Check the whole pool',
      },
    ],
  })
  async checkCompression({ all }: { all?: boolean }) {
    const spinner = createSpinner();
    spinner.start(`[Pool]: Progress 0%`);
    try {
      const { compressedSize, uncompressedSize } = await this.refCntFsckService.checkCompression(
        {
          log: (progress, count, message) => {
            spinner.text = `[Pool] - Compression at ${Math.round(Number((progress * 100n) / count))}% - ${message}`;
          },
          error: (message) => {
            spinner.fail('[Pool] - ' + message);
          },
        },
        all,
      );

      spinner.succeed(
        `[Pool]: Compression at ${Math.round(
          Number((100n * compressedSize) / uncompressedSize),
        )}% - There is ${compressedSize.toLocaleString()} compressed and ${uncompressedSize.toLocaleString()} uncompressed.`,
      );
    } catch (err) {
      spinner.fail(`[Pool]: ${(err as Error).message}`);
      console.log(err);
    }
  }
}
