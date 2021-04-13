import { createHash, Hash } from 'crypto';
import { Transform, TransformCallback, TransformOptions } from 'stream';

export class StreamHashTransform extends Transform {
  private digester: Hash;
  public hash?: Buffer;
  public length = 0;

  constructor(opts?: TransformOptions) {
    super(opts);

    this.digester = createHash('sha3-256');
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
