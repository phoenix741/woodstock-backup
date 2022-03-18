import { Injectable, Logger } from '@nestjs/common';
import { createReadStream } from 'fs';
import { AsyncIterableX } from 'ix/asynciterable';
import { filter, map, tap } from 'ix/asynciterable/operators';
import Long from 'long';
import { IMinimatch } from 'minimatch';
import { pipeline as streamPipeline, Writable } from 'stream';
import { promisify } from 'util';
import { IndexManifest } from '../manifest/index-manifest.model';
import { FileManifest } from '../models/woodstock';
import { joinBuffer } from '../utils/path.utils';
import { FileBrowserService } from './file-browser.service';
import { ChunkHashReader, FileHashReader } from './hash-reader.transform';

const pipeline = promisify(streamPipeline);

@Injectable()
export class FileReader {
  private logger = new Logger(FileReader.name);

  constructor(private browser: FileBrowserService) {}

  public getFiles(
    index: IndexManifest,
    sharePath: Buffer,
    includes: IMinimatch[] = [],
    excludes: IMinimatch[] = [],
  ): AsyncIterableX<FileManifest> {
    return this.browser
      .getFiles(sharePath)(Buffer.alloc(0), includes, excludes)
      .pipe(
        tap((file) => index.mark(file.path)),
        filter((file) => FileReader.isModified(index, file)),
        map(async (file) => {
          if (!!index.getEntry(file.path)) {
            return await this.calculateChunkHash(sharePath, file);
          } else {
            return file;
          }
        }),
      );
  }

  private static isModified(index: IndexManifest, manifest: FileManifest): boolean {
    const entry = index.getEntry(manifest.path);
    return (
      !entry ||
      !entry.manifest.stats?.lastModified?.equals(manifest.stats?.lastModified || Long.ZERO) ||
      !entry.manifest.stats?.size?.equals(manifest.stats?.size || Long.ZERO)
    );
  }

  private async calculateChunkHash(sharePath: Buffer, manifest: FileManifest): Promise<FileManifest> {
    try {
      const hashCalculator = new FileHashReader();
      const chunksCalculator = new ChunkHashReader();
      await pipeline(
        createReadStream(joinBuffer(sharePath, manifest.path)),
        hashCalculator,
        chunksCalculator,
        new Writable({
          write(_, _2, cb) {
            setImmediate(cb);
          },
        }),
      );
      manifest.sha256 = hashCalculator.hash;
      manifest.chunks = chunksCalculator.hashs;
      return manifest;
    } catch (err) {
      this.logger.warn(`Can't read hash of the file ${sharePath.toString()}/${manifest.path.toString()}`);
      return manifest;
    }
  }
}
