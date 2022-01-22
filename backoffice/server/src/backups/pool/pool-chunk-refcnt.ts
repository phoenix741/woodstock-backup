import { Injectable, Logger } from '@nestjs/common';
import {
  FileBrowserService,
  joinBuffer,
  PoolRefCount,
  ProtoPoolRefCount,
  readAllMessages,
  writeAllMessages,
} from '@woodstock/shared';
import { rename, unlink } from 'fs/promises';
import { from, reduce } from 'ix/asynciterable';
import { flatMap, map } from 'ix/asynciterable/operators';
import { join } from 'path';
import { lock } from 'proper-lockfile';
import { ApplicationConfigService } from 'src/config/application-config.service';

export type ChunkRefCnt = Record<string, number>;

@Injectable()
export class PoolChunkRefCnt {
  private logger = new Logger(PoolChunkRefCnt.name);

  constructor(private fileBrowserService: FileBrowserService, private applicationConfig: ApplicationConfigService) {}

  get poolPath() {
    return this.applicationConfig.poolPath;
  }

  /**
   * increment the reference count for each sha256 in the array.
   *
   * @param sha256s - The list of sha256s to increment.
   * @returns None
   */
  async incrBatch(sha256s: Buffer[]): Promise<void> {
    const refcntFile = join(this.poolPath, 'REFCNT');

    const unlock = await lock(refcntFile, { realpath: false });
    try {
      const result = await this.readFile(refcntFile);
      for (const sha256 of sha256s) {
        const sha256Str = sha256.toString('hex');
        const cnt = (result[sha256Str] || 0) + 1;
        result[sha256Str] = cnt;
      }
      await this.writeFile(refcntFile, result);
    } finally {
      await unlock();
    }
  }

  /**
   * decrement the reference count for each sha256 in the sha256s array.
   *
   * @param sha256s - The SHA256s of the files to decrement the reference count for.
   * @returns None
   */
  async decrBatch(sha256s: Buffer[]): Promise<void> {
    const refcntFile = join(this.poolPath, 'REFCNT');

    const unlock = await lock(refcntFile, { realpath: false });
    try {
      const result = await this.readFile(refcntFile);
      for (const sha256 of sha256s) {
        const sha256Str = sha256.toString('hex');
        const cnt = (result[sha256Str] || 0) - 1;
        if (cnt < 0) {
          this.logger.warn(`SHA256 ${sha256Str} is already pending for deletion`);
        }
        result[sha256Str] = cnt;
      }
      await this.writeFile(refcntFile, result);
    } finally {
      await unlock();
    }
  }

  /**
   * Increment the reference count of the given sha256 hash.
   *
   * @param {Buffer} sha256 - The SHA256 hash of the file to increment.
   * @returns None
   */
  async incr(sha256: Buffer): Promise<void> {
    await this.incrBatch([sha256]);
  }

  /**
   * Decrement the reference count of the given sha256 hash.
   *
   * @param {Buffer} sha256 - Buffer
   * @returns None
   */
  async decr(sha256: Buffer): Promise<void> {
    await this.decrBatch([sha256]);
  }

  /**
   * Return the list of chunks that are not referenced anymore.
   * it reads the REFCNT file and returns an async iterable of all the chunks that have a refcount of 0.
   * @returns An AsyncIterable<Buffer> that emits the sha256 of all chunks that are unused.
   */
  unusedChunks(): AsyncIterable<Buffer> {
    const poolPathBuffer = Buffer.from(this.poolPath);
    return this.fileBrowserService
      .getFilesRecursive(
        poolPathBuffer,
        (_, path) => path.name.toString() === 'REFCNT',
      )(Buffer.alloc(0))
      .pipe(
        flatMap(async (path) => {
          const filename = joinBuffer(poolPathBuffer, path).toString();
          const unlock = await lock(filename, { realpath: false });
          try {
            const result = await this.readFile(filename);
            return from(
              Object.entries(result)
                .filter(([, cnt]) => cnt <= 0)
                .map(([sha256]) => sha256),
            );
          } finally {
            await unlock();
          }
        }),
      );
  }

  private async readFile(refcntFile: string): Promise<ChunkRefCnt> {
    const it = readAllMessages<PoolRefCount>(refcntFile, ProtoPoolRefCount);
    return await reduce(it, {
      seed: {} as ChunkRefCnt,
      callback: async (acc, msg) => {
        acc[msg.message.sha256.toString('hex')] = msg.message.refCount;
        return acc;
      },
    });
  }

  private async writeFile(refcntFile: string, chunkRefCnt: ChunkRefCnt): Promise<void> {
    const it = from(Object.entries(chunkRefCnt)).pipe(
      map(([sha256, refCount]) => ({
        sha256: Buffer.from(sha256, 'hex'),
        refCount,
      })),
    );
    const refcntFileNew = refcntFile + '.new';
    try {
      await writeAllMessages(refcntFileNew, ProtoPoolRefCount, it);
      await unlink(refcntFile).catch(() => undefined);
      await rename(refcntFileNew, refcntFile);
    } finally {
      await unlink(refcntFileNew).catch(() => undefined);
    }
  }
}
