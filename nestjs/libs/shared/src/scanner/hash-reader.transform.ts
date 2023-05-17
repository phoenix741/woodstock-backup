import { CHUNK_SIZE, HASH_ALGO } from '@woodstock/core';
import { createHash } from 'crypto';
import { Transform, TransformCallback, TransformOptions } from 'stream';

export class FileHashReader extends Transform {
  #digester = createHash(HASH_ALGO);
  hash?: Buffer;
  length = 0n;

  constructor(opts?: TransformOptions) {
    super(opts);
  }

  _transform(chunk: Buffer | string, enc: BufferEncoding, cb: TransformCallback): void {
    const buffer = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk, enc);

    this.#digester.update(buffer);
    this.length += BigInt(buffer.length);

    this.push(buffer);
    cb();
  }

  _flush(cb: TransformCallback): void {
    this.hash = this.#digester.digest();
    cb();
  }
}

export class ChunkHashReader extends Transform {
  #digester = createHash(HASH_ALGO);
  hashs: Buffer[] = [];
  bufferLength = 0;

  constructor(opts?: TransformOptions) {
    super(opts);
  }

  _transform(chunk: Buffer | string, enc: BufferEncoding, cb: TransformCallback): void {
    const buffer = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk, enc);

    const chunkSizeRest = CHUNK_SIZE - this.bufferLength;
    let shaData, shaDataRest;
    if (buffer.length >= chunkSizeRest) {
      shaData = buffer.slice(0, chunkSizeRest);
      shaDataRest = buffer.slice(chunkSizeRest);
    } else {
      shaData = buffer;
      shaDataRest = Buffer.alloc(0);
    }

    this.#digester.update(shaData);
    this.bufferLength += shaData.length;

    if (this.bufferLength >= CHUNK_SIZE) {
      this.hashs.push(this.#digester.digest());
      this.#digester = createHash(HASH_ALGO);
      this.#digester.update(shaDataRest);
      this.bufferLength = shaDataRest.length;
    }

    this.push(buffer);
    cb();
  }

  _flush(cb: TransformCallback): void {
    if (this.bufferLength) {
      this.hashs.push(this.#digester.digest());
    }

    cb();
  }
}
