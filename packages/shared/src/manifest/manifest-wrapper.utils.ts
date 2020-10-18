import { createReadStream, createWriteStream } from 'fs';
import { asAsyncIterable } from 'ix';
import * as mkdirp from 'mkdirp';
import { dirname } from 'path';
import { Type } from 'protobufjs';
import { pipeline } from 'stream/promises';
import { ProtobufMessageReader, ProtobufMessageWithPosition } from './protobuf-message-reader.utils';
import { ProtobufMessageWriter } from './protobuf-message-writer.utils';

export function readAllMessages<T>(path: string, type: Type): AsyncIterable<ProtobufMessageWithPosition<T>> {
  const reader = createReadStream(path);
  const transform = new ProtobufMessageReader(type);

  reader.pipe(transform);
  reader.on('error', (err) => {
    transform.emit('error', err);
    transform.end();
    reader.close();
  });
  transform.on('error', () => {
    transform.end();
    reader.close();
  });

  return transform.pipe(asAsyncIterable<ProtobufMessageWithPosition<T>>({ objectMode: true }));
}

export async function writeAllMessages<O>(path: string, type: Type, source: AsyncIterable<O>): Promise<void> {
  const streamPath = path;

  await mkdirp(dirname(streamPath));

  const stream = createWriteStream(streamPath, { flags: 'a' });
  const transform = new ProtobufMessageWriter<O>(type);

  return await pipeline(source, transform, stream);
}
