import { Injectable, Logger } from '@nestjs/common';
import { createReadStream } from 'fs';
import { defer, iif, Observable, of } from 'rxjs';
import { concatMap, filter } from 'rxjs/operators';
import { pipeline as streamPipeline, Writable } from 'stream';
import { promisify } from 'util';

import { IndexManifest } from '../manifest/index-manifest.model';
import { FileManifest } from '../models';
import { joinBuffer } from '../utils/path.utils';
import { FileBrowserService } from './file-browser.service';
import { ChunkHashReader, FileHashReader } from './hash-reader.transform';
import { Manifest } from '../manifest/manifest.model';

const pipeline = promisify(streamPipeline);

@Injectable()
export class FileReader {
  private logger = new Logger(FileReader.name);

  constructor(private browser: FileBrowserService) {}

  public getFiles(
    index: IndexManifest,
    sharePath: Buffer,
    includes: RegExp[] = [],
    excludes: RegExp[] = [],
  ): Observable<FileManifest> {
    return this.browser
      .getFiles(sharePath)(Buffer.alloc(0), includes, excludes)
      .pipe(
        filter((file) => FileBrowserService.isRegularFile(file.stats?.mode)),
        filter((file) => FileReader.isModified(index, file)),
        concatMap((file) =>
          iif(
            () => !!index.getEntry(file.path),
            defer(() => this.calculateChunkHash(sharePath, file)),
            of(file),
          ),
        ),
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
