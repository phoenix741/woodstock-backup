import { Logger } from '@nestjs/common';
import { createReadStream, createWriteStream } from 'fs';
import { cp, rename, unlink } from 'fs/promises';
import { never, pipe } from 'ix/asynciterable';
import * as mkdirp from 'mkdirp';
import { dirname } from 'path';
import { Type } from 'protobufjs';
import * as stream from 'stream';
import { Duplex, Readable, Stream } from 'stream';
import { pipeline } from 'stream/promises';
import { createDeflate, createInflate } from 'zlib';
import { notUndefined } from '../utils';
import { fromNodeStream } from '../utils/fromnodestream';
import { tmpNameAsync } from '../utils/path.utils';
import { ProtobufMessageReader, ProtobufMessageWithPosition } from './transform/protobuf-message-reader.utils';
import { ProtobufMessageWriter } from './transform/protobuf-message-writer.utils';

// eslint-disable-next-line @typescript-eslint/ban-types
const compose: (...streams: Array<Stream | Iterable<unknown> | AsyncIterable<unknown> | Function>) => Duplex = (
  stream as any
).compose;

export class ProtobufService {
  private logger = new Logger(ProtobufService.name);

  /**
   * it reads a file and emits Protobuf messages.
   *
   * @param {string} path - The path to the file to read.
   * @param {Type} type - Type
   * @returns An AsyncIterable<ProtobufMessageWithPosition<T>>.
   */
  loadFile<T>(path: string, type: Type, compress = true): AsyncIterable<ProtobufMessageWithPosition<T>> {
    this.logger.debug(`Read the file ${path} with type ${type.name}`);
    try {
      const reader = createReadStream(path);
      const transform = new ProtobufMessageReader(type);

      const streams = [reader, ...(compress ? [createInflate()] : []), transform];

      return fromNodeStream(compose(...streams));
    } catch (err) {
      this.logger.error(`Can't read the file ${path} with type ${type.name}`, err);
      return never();
    }
  }

  /**
   * It writes a file with the given path, type and source.
   * The file will be written in protobuf format.
   * The content will be appended to the file.
   *
   * @param {string} path - The path to the file to write.
   * @param {Type} type - The type of the message to write.
   * @param source - The source of the data to write.
   * @returns A promise that resolves to the real filename.
   */
  async writeFile<O>(path: string, type: Type, source: AsyncIterable<O>, compress = true): Promise<void> {
    this.logger.debug(`Write the file ${path} with type ${type.name}`);

    await mkdirp(dirname(path));

    const stream = createWriteStream(path, { flags: 'a' });
    const transform = new ProtobufMessageWriter<O>(type);

    const allDefinedSource = pipe(source, notUndefined());

    const streams = [Readable.from(allDefinedSource), transform, ...(compress ? [createDeflate()] : []), stream];

    await pipeline(streams);
  }

  i = 0;
  /**
   * it takes an AsyncIterable of objects of type O, and writes them to a file with the given path.
   * The file will be written in protobuf format.
   * A temporary file will be created, and the content of the file will be written to it.
   *
   * @param {string} path - The path to the file to write.
   * @param {Type} type - The type of the message to write.
   * @param source - The source of the data to write.
   * @returns None
   */
  async atomicWriteFile<O>(path: string, type: Type, source: AsyncIterable<O>, compress = true): Promise<void> {
    this.logger.debug(`Atomic write the file ${path} with type ${type.name}`);

    const tmpFilename = await tmpNameAsync({
      tmpdir: dirname(path),
    });

    try {
      await this.writeFile(tmpFilename, type, source, compress);
      await rename(tmpFilename, path);
      //await cp(path, path + '-' + this.i++);
    } catch (err) {
      await this.rmFile(tmpFilename);
      throw err;
    }
  }

  /**
   * Remove the file after determining the extension of the file
   * @param path the file to remove
   */
  async rmFile(path: string): Promise<void> {
    await unlink(path).catch((err) => {
      if (err.code !== 'ENOENT') {
        // Ignore error on unlink, the file don't exist
        throw err;
      }
    });
  }
}
