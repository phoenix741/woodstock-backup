import { FsckService, PoolService } from '@woodstock/shared';
import { Command, Console, createSpinner } from 'nestjs-console';
import { pipeline } from 'stream/promises';

@Console({
  command: 'pool',
})
export class PoolCommand {
  constructor(private refCntFsckService: FsckService, private poolService: PoolService) {}

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
    try {
      const errorCount = await this.refCntFsckService.processRefcnt(
        {
          log: (progress, count, message) => {
            spinner.text = `[Pool] - Progress ${Math.round((progress * 100) / count)}% - ${message}`;
          },
          error: (message) => {
            spinner.fail('[Pool] - ' + message);
          },
        },
        !fix,
      );

      const { inUnused, inRefcnt, inNothing, missing } = await this.refCntFsckService.processUnused(
        {
          log: (progress, count, message) => {
            spinner.text = `[Pool] - Progress ${Math.round((progress * 100) / count)}% - ${message}`;
          },
          error: (message) => {
            spinner.fail('[Pool] - ' + message);
          },
        },
        !fix,
      );

      spinner.succeed(
        `[Pool]: Reference count validation is finished. ${
          errorCount + missing + inNothing
        } errors found.  ${inUnused} chunks to delete, ${inRefcnt} in refcnt.`,
      );
    } catch (err) {
      spinner.fail(`[Pool]: ${(err as Error).message}`);
      console.log(err);
    }
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
    try {
      const errorCount = await this.refCntFsckService.processRefcnt(
        {
          log: (progress, count, message) => {
            spinner.text = `[Pool] - Progress ${Math.round((progress * 100) / count)}% - ${message}`;
          },
          error: (message) => {
            spinner.fail('[Pool] - ' + message);
          },
        },
        !fix,
      );

      spinner.succeed(`[Pool]: Reference count validation is finished. ${errorCount} errors found.`);
    } catch (err) {
      spinner.fail(`[Pool]: ${(err as Error).message}`);
      console.log(err);
    }
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
    try {
      const { inUnused, inRefcnt, inNothing, missing } = await this.refCntFsckService.processUnused(
        {
          log: (progress, count, message) => {
            spinner.text = `[Pool] - Progress ${Math.round((progress * 100) / count)}% - ${message}`;
          },
          error: (message) => {
            spinner.fail('[Pool] - ' + message);
          },
        },
        !fix,
      );

      spinner.succeed(
        `[Pool]: Reference count validation is finished. ${inUnused} unused chunks, ${inRefcnt} in refcnt, ${inNothing} in nothing, ${missing} in missing.`,
      );
    } catch (err) {
      spinner.fail(`[Pool]: ${(err as Error).message}`);
      console.log(err);
    }
  }

  @Command({
    command: 'remove-unused',
    description: 'Remove unused chunks',
  })
  async removeUnused() {
    const spinner = createSpinner();
    let count = 0;
    spinner.start(`[Unused]: Progress 0 chunk(s)`);

    await new Promise<void>((resolve, reject) => {
      this.poolService.removeUnusedFiles().subscribe({
        next: (chunk) => {
          spinner.text = `[Unused] - Progress ${++count} chunk(s) : ${chunk.sha256.toString('hex')}`;
        },
        error: (err) => {
          spinner.fail(`[Unused]: ${(err as Error).message}`);
          reject(err);
        },
        complete: () => {
          spinner.succeed(`[Unused]: Reference count validation is finished. ${count} chunk removed.`);
          resolve();
        },
      });
    });
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
}
