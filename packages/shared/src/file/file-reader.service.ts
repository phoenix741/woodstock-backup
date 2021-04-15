import { Injectable } from '@nestjs/common';
import { createReadStream } from 'fs';
import { Observable } from 'rxjs';
import { filter, mergeMap, concatMap } from 'rxjs/operators';
import { FileManifest } from 'src/models';
import { pipeline as streamPipeline } from 'stream';
import { promisify } from 'util';

import { IndexManifest } from '../manifest/index-manifest.model';
import { joinBuffer } from '../utils/path.utils';
import { FileBrowserService } from './file-browser.service';
import { ChunkHashReader, FileHashReader } from './hash-reader.transform';

const pipeline = promisify(streamPipeline);

Injectable();
export class FileReader {
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
        concatMap((file) => FileReader.calculateChunkHash(sharePath, file)),
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

  private static async calculateChunkHash(sharePath: Buffer, file: FileManifest): Promise<FileManifest> {
    const hashCalculator = new FileHashReader();
    const chunksCalculator = new ChunkHashReader();
    await pipeline(createReadStream(joinBuffer(sharePath, file.path)), hashCalculator, chunksCalculator);
    file.sha256 = hashCalculator.hash;
    file.chunks = chunksCalculator.hashs;
    return file;
  }
}
