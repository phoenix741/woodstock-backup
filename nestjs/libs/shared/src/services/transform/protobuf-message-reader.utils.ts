import { Reader, Type } from 'protobufjs';
import { Transform, TransformCallback, TransformOptions } from 'stream';

export interface ProtobufMessageWithPosition<T> {
  position: number;
  message: T;
}

export class ProtobufMessageReader extends Transform {
  private remainder = Buffer.alloc(0);
  private _position = 0;

  constructor(private type: Type, opts?: TransformOptions) {
    super({
      objectMode: true,
      readableObjectMode: true,
      ...opts,
    });
  }

  get position(): number {
    return this._position;
  }

  _transform(chunk: Buffer, enc: BufferEncoding, cb: TransformCallback): void {
    const buffer = Buffer.concat([this.remainder, chunk]);

    let lastPos = 0;
    const reader = Reader.create(buffer);
    try {
      while (reader.pos < reader.len) {
        lastPos = reader.pos;
        const frame = this.type.decodeDelimited(reader);
        this.push({ position: this.position, message: this.type.toObject(frame) });

        this._position += reader.pos - lastPos;
      }

      this.remainder = Buffer.alloc(0);
    } catch (err) {
      this.remainder = buffer.slice(lastPos);
    }

    cb();
  }
}
