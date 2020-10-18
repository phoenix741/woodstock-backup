import { join } from 'path';
import { loadSync, Reader, Type } from 'protobufjs';
import { Transform, TransformCallback, TransformOptions } from 'stream';

const root = loadSync(join(__dirname, 'woodstock.proto'));

export const ProtoFileManifest = root.lookupType('woodstock.FileManifest');
export const ProtoFileManifestJournalEntry = root.lookupType('woodstock.FileManifestJournalEntry');
export const ProtoPrepareBackupRequest = root.lookupType('woodstock.PrepareBackupRequest');
export const ProtoPrepareBackupReply = root.lookupType('woodstock.PrepareBackupReply');
export const ProtoLaunchBackupRequest = root.lookupType('woodstock.LaunchBackupRequest');
export const ProtoGetChunkRequest = root.lookupType('woodstock.GetChunkRequest');
export const ProtoFileChunk = root.lookupType('woodstock.FileChunk');

export interface ProtobufMessageWithPosition<T> {
  position: number;
  message: T;
}

export class ProtobufMessageReader extends Transform {
  private remainder = Buffer.alloc(0);
  private _position = 0;

  constructor(private type: Type, opts?: TransformOptions, private varint = true) {
    super({
      objectMode: false,
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
        const frame = this.varint ? this.type.decodeDelimited(reader) : this.type.decode(reader, reader.fixed32());
        this.push({ position: this._position, message: this.type.toObject(frame) });

        this._position += reader.pos - lastPos;
      }

      this.remainder = Buffer.alloc(0);
    } catch (err) {
      this.remainder = buffer.slice(lastPos);
    }

    cb();
  }

  _flush(cb: TransformCallback): void {
    cb();
  }
}
