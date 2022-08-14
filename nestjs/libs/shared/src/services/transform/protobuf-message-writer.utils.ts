import type { Type } from 'protobufjs';
import { Writer } from 'protobufjs';
import { Transform, TransformCallback, TransformOptions } from 'stream';

const WRITE_BUFFER_SIZE = Math.pow(2, 16);

export class ProtobufMessageWriter<T> extends Transform {
  private grpcWriter = new Writer();

  constructor(private type: Type, opts?: TransformOptions) {
    super({
      objectMode: false,
      writableObjectMode: true,
      ...opts,
    });
  }

  private pushData() {
    const data = this.grpcWriter.finish();
    this.grpcWriter.reset();
    this.push(data);
  }

  _transform(chunk: T, enc: BufferEncoding, cb: TransformCallback): void {
    this.type.encodeDelimited(chunk, this.grpcWriter);
    if (this.grpcWriter.len > WRITE_BUFFER_SIZE) {
      this.pushData();
    }

    cb();
  }

  _flush(cb: TransformCallback): void {
    this.pushData();

    cb();
  }
}
