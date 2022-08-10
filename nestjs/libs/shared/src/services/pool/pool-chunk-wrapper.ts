import { Logger } from '@nestjs/common';
import * as assert from 'assert';
import { createReadStream, createWriteStream } from 'fs';
import { rename, stat } from 'fs/promises';
import * as mkdirp from 'mkdirp';
import { join } from 'path';
import * as stream from 'stream';
import { Duplex, pipeline as streamPipeline, Readable, Stream, Writable } from 'stream';
import * as util from 'util';
import { createDeflate, createInflate } from 'zlib';
import { CHUNK_SIZE } from '../../constants';
import { FileHashReader } from '../../file/hash-reader.transform';
import { PoolChunkInformation } from '../../models/pool-chunk.dto';
import { getTemporaryFileName, isExists, rm } from '../../utils';
import { calculateChunkDir } from './pool-chunk.utils';
import { PoolService } from './pool.service';

const pipeline = util.promisify(streamPipeline);

// eslint-disable-next-line @typescript-eslint/ban-types
const compose: (...streams: Array<Stream | Iterable<unknown> | AsyncIterable<unknown> | Function>) => Duplex = (
  stream as any
).compose;

export class PoolChunkWrapper {
  private logger = new Logger(PoolChunkWrapper.name);

  private sha256Str?: string;

  constructor(private poolService: PoolService, private poolPath: string, private _sha256?: Buffer) {
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
  static async exists(service: PoolService, poolPath: string, sha256: Buffer): Promise<boolean> {
    const wrapper = new PoolChunkWrapper(service, poolPath, sha256);
    return await wrapper.exists();
  }

  /**
   * Get the chunk file (read only), if the file exists.
   * @throw exception if the the file doesn't exists.
   * @returns the wrapper that can be used do read the file.
   */
  static get(service: PoolService, poolPath: string, sha256?: Buffer): PoolChunkWrapper {
    return new PoolChunkWrapper(service, poolPath, sha256);
  }

  async exists(): Promise<boolean> {
    return await isExists(this.chunkPath);
  }

  read(): Readable {
    return compose(createReadStream(this.chunkPath), createInflate());
  }

  /**
   * Get chunk information by reading the chunk.
   *
   * WARNING: This method read the chunk from the disk and is slow compared to getting chunk information
   * from the pool.
   * @returns
   */
  async getChunkInformation(): Promise<PoolChunkInformation> {
    const chunkStatistics = await stat(this.chunkPath, { bigint: true });

    const nullStream = new Writable({
      write(_, _2, callback) {
        setImmediate(callback);
      },
    });

    const hashCalculator = new FileHashReader();
    await pipeline(createReadStream(this.chunkPath), createInflate(), hashCalculator, nullStream);
    if (!this.sha256 || !hashCalculator.hash?.equals(this.sha256)) {
      this.logger.error(
        `When reading chunk, the hash should be ${this.sha256Str} but is ${hashCalculator.hash?.toString('hex')}`,
      );
    }

    assert.ok(!!hashCalculator.hash, `Hash of the file ${this.chunkPath} shouldn't be undefined`);
    return { sha256: hashCalculator.hash, compressedSize: chunkStatistics.size, size: hashCalculator.length };
  }

  async write(inputStream: Readable, debugFilename: string): Promise<PoolChunkInformation> {
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
          `When writing chunk (for file ${debugFilename}), the hash should be ${
            this.sha256Str
          } but is ${hashCalculator.hash?.toString('hex')}`,
        );
      }

      // Rename
      assert.ok(!!hashCalculator.hash, `Hash of the file ${this.chunkPath} shouldn't be undefined`);
      this.sha256 = hashCalculator.hash;
      const chunkStatistics = await stat(tempfilename, { bigint: true });
      const chunk = {
        sha256: this.sha256,
        size: hashCalculator.length,
        compressedSize: chunkStatistics.size,
      };

      if (await this.exists()) {
        this.logger.debug(`The chunk ${this.sha256Str} is already present`);
        await rm(tempfilename);
      } else {
        await mkdirp(this.chunkDir);
        await rename(tempfilename, this.chunkPath);
      }

      return chunk;
    } catch (err) {
      await rm(tempfilename);
      throw err;
    }
  }

  async remove(): Promise<void> {
    await rm(this.chunkPath);
  }

  private get chunkDir() {
    if (this.sha256Str) {
      return calculateChunkDir(this.poolPath, this.sha256Str);
    } else {
      return join(this.poolPath, '_new');
    }
  }

  private get chunkPath() {
    const filename = this.sha256Str ? this.sha256Str : getTemporaryFileName();
    return join(this.chunkDir, `${filename}-sha256.zz`);
  }
}
