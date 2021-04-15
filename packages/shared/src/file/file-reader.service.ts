import { Injectable, Logger } from '@nestjs/common';
import { createReadStream } from 'fs';
import { Observable } from 'rxjs';
import { concatMap, filter } from 'rxjs/operators';
import { FileManifest } from 'src/models';
import { pipeline as streamPipeline, Writable } from 'stream';
import { promisify } from 'util';

import { IndexManifest } from '../manifest/index-manifest.model';
import { joinBuffer } from '../utils/path.utils';
import { FileBrowserService } from './file-browser.service';
import { ChunkHashReader, FileHashReader } from './hash-reader.transform';

const pipeline = promisify(streamPipeline);

Injectable();
export class FileReader {
  private logger = new Logger(FileReader.name);

  constructor(private browser: FileBrowserService) {}

  public getFiles(
    index: IndexManifest,
    sharePath: Buffer,
    includes: RegExp[],
    excludes: RegExp[],
  ): Observable<FileManifest> {
    return this.browser
      .getFiles(sharePath)(Buffer.alloc(0), includes, excludes)
      .pipe(
        filter((file) => FileBrowserService.isRegularFile(file.stats?.mode)),
        filter((file) => FileReader.isModified(index, file)),
        concatMap((file) => this.calculateChunkHash(sharePath, file)), // FIXME: Don't parse file if not modified
      );
  }

  private static isModified(index: IndexManifest, manifest: FileManifest): boolean {
    const entry = index.getEntry(manifest.path);
    return (
      !entry ||
      entry.manifest.stats?.lastModified !== manifest.stats?.lastModified ||
      entry.manifest.stats?.size !== manifest.stats?.size
    );
  }

  private async calculateChunkHash(sharePath: Buffer, file: FileManifest): Promise<FileManifest> {
    try {
      const hashCalculator = new FileHashReader();
      const chunksCalculator = new ChunkHashReader();
      await pipeline(
        createReadStream(joinBuffer(sharePath, file.path)),
        hashCalculator,
        chunksCalculator,
        new Writable({
          write(_, _2, cb) {
            setImmediate(cb);
          },
        }),
      );
      file.sha256 = hashCalculator.hash;
      file.chunks = chunksCalculator.hashs;
      return file;
    } catch (err) {
      this.logger.warn(`Can't read hash of the file ${sharePath.toString()}/${file.path.toString()}`);
      return file;
    }
  }
}
