import { Logger } from '@nestjs/common';
import { CHUNK_SIZE, FileHashReader, getTemporaryFileName } from '@woodstock/shared';
import * as assert from 'assert';
import { constants, createReadStream, createWriteStream } from 'fs';
import { access, rename, rm, stat } from 'fs/promises';
import * as mkdirp from 'mkdirp';
import { join } from 'path';
import { pipeline as streamPipeline, Readable, Writable } from 'stream';
import * as util from 'util';
import { createDeflate, createInflate } from 'zlib';
import { PoolChunkInformation } from './pool-chunk.dto';

const pipeline = util.promisify(streamPipeline);

export class PoolChunkWrapper {
  private logger = new Logger(PoolChunkWrapper.name);

  private sha256Str?: string;

  constructor(private poolPath: string, private _sha256?: Buffer) {
    this.sha256 = _sha256;
  }

  public get sha256(): Buffer | undefined {
    return this._sha256;
  }

  public set sha256(value: Buffer | undefined) {
    this._sha256 = value;
    this.sha256Str = value?.toString('hex');
  }

  /**
   * Check if the chunk exists in the pool.
   * @returns true if the file exists
   */
  static async exists(poolPath: string, sha256: Buffer): Promise<boolean> {
    const wrapper = new PoolChunkWrapper(poolPath, sha256);
    return await wrapper.exists();
  }

  /**
   * Get the chunk file (read only), if the file exists.
   * @throw exception if the the file doesn't exists.
   * @returns the wrapper that can be used do read the file.
   */
  static get(poolPath: string, sha256?: Buffer): PoolChunkWrapper {
    return new PoolChunkWrapper(poolPath, sha256);
  }

  async exists(): Promise<boolean> {
    return access(this.chunkPath, constants.F_OK)
      .then(() => true)
      .catch(() => false);
  }

  async read(outputStream: Writable): Promise<PoolChunkInformation> {
    const chunkStatistics = await stat(this.chunkPath, { bigint: true });

    const hashCalculator = new FileHashReader();
    await pipeline(createReadStream(this.chunkPath), createInflate(), hashCalculator, outputStream);
    if (hashCalculator.hash !== this.sha256) {
      this.logger.error(
        `When reading chunk, the hash should be ${this.sha256Str} but is ${hashCalculator.hash?.toString('hex')}`,
      );
    }

    assert.ok(!!hashCalculator.hash, `Hash of the file ${this.chunkPath} shouldn't be undefined`);
    return { sha256: hashCalculator.hash, compressedSize: chunkStatistics.size, size: hashCalculator.length };
  }

  async write(inputStream: Readable): Promise<PoolChunkInformation> {
    await mkdirp(this.chunkDir);

    const tempfilename = join(this.chunkDir, getTemporaryFileName());

    try {
      const hashCalculator = new FileHashReader();
      await pipeline(inputStream, hashCalculator, createDeflate(), createWriteStream(tempfilename));
      if (CHUNK_SIZE < hashCalculator.length) {
        this.logger.error(`Chunk ${this.sha256Str} has not the right size length: ${hashCalculator.length}`);
      }

      if (this.sha256 && !hashCalculator.hash?.equals(this.sha256)) {
        this.logger.error(
          `When writing chunk, the hash should be ${this.sha256Str} but is ${hashCalculator.hash?.toString('hex')}`,
        );
      }

      // Rename
      assert.ok(!!hashCalculator.hash, `Hash of the file ${this.chunkPath} shouldn't be undefined`);
      this.sha256 = hashCalculator.hash;
      const chunkStatistics = await stat(tempfilename, { bigint: true });

      if (await this.exists()) {
        this.logger.debug(`The chunk ${this.sha256Str} is already present`);
        await rm(tempfilename);
      } else {
        await mkdirp(this.chunkDir);
        await rename(tempfilename, this.chunkPath);
      }

      return {
        sha256: this.sha256,
        size: hashCalculator.length,
        compressedSize: chunkStatistics.size,
      };
    } catch (err) {
      try {
        await rm(tempfilename);
      } catch (_) {}
      throw err;
    }
  }

  private get chunkDir() {
    if (this.sha256Str) {
      const part1 = this.sha256Str.substr(0, 2);
      const part2 = this.sha256Str.substr(2, 2);
      const part3 = this.sha256Str.substr(4, 2);

      return join(this.poolPath, part1, part2, part3);
    } else {
      return join(this.poolPath, '_new');
    }
  }

  private get chunkPath() {
    const filename = this.sha256Str ? this.sha256Str : getTemporaryFileName();
    return join(this.chunkDir, `${filename}-sha256.zz`);
  }
}
