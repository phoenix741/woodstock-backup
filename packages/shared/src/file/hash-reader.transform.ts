import { createHash, Hash } from 'crypto';
import { Transform, TransformCallback, TransformOptions } from 'stream';

export interface HashReaderOptions {
  intermediateHash?: boolean;
}

export class HashReader extends Transform {
  private digester = createHash('sha3-256');
  public hash?: Buffer;
  public length = 0;

  constructor(opts?: TransformOptions & HashReaderOptions) {
    super(opts);
  }

  _transform(chunk: Buffer | string, enc: BufferEncoding, cb: TransformCallback): void {
    const buffer = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk, enc);

    this.digester.update(buffer);
    this.length += buffer.length;

    this.push(buffer);
    cb();
  }

  _flush(cb: TransformCallback): void {
    this.hash = this.digester.digest();
    cb();
  }
}
