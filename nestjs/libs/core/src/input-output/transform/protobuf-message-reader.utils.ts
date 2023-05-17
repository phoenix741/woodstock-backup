import type { Type } from 'protobufjs';
import { Reader } from 'protobufjs';
import { Transform, TransformCallback, TransformOptions } from 'stream';

export class ProtobufMessageReader extends Transform {
  #remainder = Buffer.alloc(0);
  #position = 0;

  constructor(private type: Type, opts?: TransformOptions) {
    super({
      objectMode: true,
      readableObjectMode: true,
      ...opts,
    });
  }

  get position(): number {
    return this.#position;
  }

  _transform(chunk: Buffer, enc: BufferEncoding, cb: TransformCallback): void {
    const buffer = Buffer.concat([this.#remainder, chunk]);

    let lastPos = 0;
    const reader = Reader.create(buffer);
    try {
      while (reader.pos < reader.len) {
        lastPos = reader.pos;
        const frame = this.type.decodeDelimited(reader);
        this.push({ position: this.position, message: this.type.toObject(frame) });

        this.#position += reader.pos - lastPos;
      }

      this.#remainder = Buffer.alloc(0);
    } catch (err) {
      this.#remainder = buffer.slice(lastPos);
    }

    cb();
  }
}
