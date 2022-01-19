import { Injectable, Logger } from '@nestjs/common';
import { FileBrowserService, joinBuffer } from '@woodstock/shared';
import { from } from 'ix/asynciterable';
import { flatMap } from 'ix/asynciterable/operators';
import { join } from 'path';
import { lock } from 'proper-lockfile';
import { ApplicationConfigService } from 'src/config/application-config.service';
import { YamlService } from 'src/utils/yaml.service';
import { calculateChunkDir } from './pool-chunk.utils';

export type ChunkRefCnt = Record<string, number>;

@Injectable()
export class PoolChunkRefCnt {
  private logger = new Logger(PoolChunkRefCnt.name);

  constructor(
    private yamlService: YamlService,
    private fileBrowserService: FileBrowserService,
    private applicationConfig: ApplicationConfigService,
  ) {}

  get poolPath() {
    return this.applicationConfig.poolPath;
  }

  /**
   * Increment the number of reference to the chunk by batch.
   *
   * @param sha256s Sha256 of the chunks
   */
  async incrBatch(sha256s: Buffer[]): Promise<void> {
    this.logger.log(`Incrementing refcnt for ${sha256s.length} chunks`);
    await Promise.all(sha256s.map(async (sha256) => this.incr(sha256)));
  }

  /**
   * Decrement the number of reference to the chunk by batch.
   *
   * @param sha256s Sha256 of the chunks
   */
  async decrBatch(sha256s: Buffer[]): Promise<void> {
    this.logger.log(`Incrementing refcnt for ${sha256s.length} chunks`);
    await Promise.all(sha256s.map(async (sha256) => this.decr(sha256)));
  }

  /**
   * Increment the number of references to the chunk.
   *
   * Lock the refcnt file before updating the refcnt.
   *
   * @param sha256 Sha256 of the chunk
   * @returns number of references to the chunk
   */
  async incr(sha256: Buffer): Promise<number> {
    this.logger.log(`Incrementing refcnt for chunk ${sha256.toString('hex')}`);
    const { refcntFile, sha256Str } = this.getRefCntFileName(sha256);
    const unlock = await lock(refcntFile, { realpath: false });
    try {
      const result = await this.readFile(refcntFile);
      const cnt = (result[sha256Str] || 0) + 1;
      result[sha256Str] = cnt;
      await this.writeFile(refcntFile, result);
      return cnt;
    } finally {
      await unlock();
    }
  }

  /**
   * Decerement the number of references to the chunk.
   *
   * Lock the refcnt file before updating the refcnt.
   *
   * @param sha256 Sha256 of the chunk
   * @returns number of references to the chunk
   */
  async decr(sha256: Buffer): Promise<number> {
    this.logger.log(`Decrementing refcnt for chunk ${sha256.toString('hex')}`);
    const { refcntFile, sha256Str } = this.getRefCntFileName(sha256);
    const unlock = await lock(refcntFile, { realpath: false });
    try {
      const result = await this.readFile(refcntFile);
      const cnt = (result[sha256Str] || 0) - 1;
      if (cnt < 0) {
        this.logger.warn(`SHA256 ${sha256Str} is already pending for deletion`);
      }
      result[sha256Str] = cnt;
      await this.writeFile(refcntFile, result);
      return cnt;
    } finally {
      await unlock();
    }
  }

  /**
   * Return the list of chunks that are not referenced anymore.
   *
   * @returns List of chunks
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
    return await this.yamlService.loadFile<ChunkRefCnt>(refcntFile, {});
  }

  private async writeFile(refcntFile: string, chunkRefCnt: ChunkRefCnt): Promise<void> {
    await this.yamlService.writeFile(refcntFile, chunkRefCnt);
  }

  private getRefCntFileName(sha256: Buffer) {
    const sha256Str = sha256.toString('hex');
    const chunkDir = calculateChunkDir(this.poolPath, sha256Str);
    return {
      sha256Str,
      refcntFile: join(chunkDir, 'REFCNT'),
    };
  }
}
