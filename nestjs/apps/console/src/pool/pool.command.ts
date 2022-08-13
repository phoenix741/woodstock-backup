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
    let count = 0;
    spinner.start(`[Unused]: Progress 0 chunk(s)`);

    await new Promise<void>((resolve, reject) => {
      this.poolService.removeUnusedFiles(target).subscribe({
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

  @Command({
    command: 'verify-chunk',
    description: 'Verify the integrity of all chunk',
  })
  async verifyChunks() {
    const spinner = createSpinner();
    spinner.start(`[Pool]`);
    try {
      const { chunkOk, chunkKo } = await this.refCntFsckService.processVerifyChunk({
        log: (progress, count, message) => {
          spinner.text = `[Pool] - ${progress} - ${message}`;
        },
        error: (message) => {
          spinner.fail('[Pool] - ' + message);
        },
      });

      spinner.succeed(`[Pool]: There is ${chunkOk} chunk validated and ${chunkKo} with the wrong sha256.`);
    } catch (err) {
      spinner.fail(`[Pool]: ${(err as Error).message}`);
      console.log(err);
    }
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
