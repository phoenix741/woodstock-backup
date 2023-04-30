import { Injectable, Logger } from '@nestjs/common';
import { createReadStream } from 'fs';
import { AsyncIterableX } from 'ix/asynciterable';
import { filter, map, tap } from 'ix/asynciterable/operators';
import Long from 'long';
import type { IMinimatch } from 'minimatch';
import { pipeline as streamPipeline, Writable } from 'stream';
import { promisify } from 'util';
import { ManifestService } from '../manifest';
import { IndexManifest } from '../manifest/index-manifest.model.js';
import { FileManifest, FileManifestJournalEntry } from '../shared';
import { longToBigInt } from '../utils';
import { joinBuffer } from '../utils/path.utils.js';
import { FileBrowserService } from './file-browser.service.js';
import { ChunkHashReader, FileHashReader } from './hash-reader.transform.js';

const pipeline = promisify(streamPipeline);

function isModified(index: IndexManifest, manifest: FileManifest): boolean {
  const entry = index.getEntry(manifest.path);
  const isModified =
    !entry ||
    !entry.manifest.stats?.lastModified?.equals(manifest.stats?.lastModified || Long.ZERO) ||
    !entry.manifest.stats?.size?.equals(manifest.stats?.size || Long.ZERO);
  return isModified;
}

@Injectable()
export class FileReaderService {
  #logger = new Logger(FileReaderService.name);

  constructor(private browser: FileBrowserService) {}

  getFiles(
    index: IndexManifest,
    sharePath: Buffer,
    includes: IMinimatch[] = [],
    excludes: IMinimatch[] = [],
  ): AsyncIterableX<FileManifestJournalEntry> {
    return this.browser
      .getFiles(sharePath)(Buffer.alloc(0), includes, excludes)
      .pipe(
        tap((file) => index.mark(file.path)),
        filter((file) => isModified(index, file)),
        map(async (file) => {
          if (
            !!index.getEntry(file.path) &&
            FileBrowserService.isRegularFile(longToBigInt(file.stats?.mode || Long.ZERO))
          ) {
            const manifest = await this.#calculateChunkHash(sharePath, file);
            return ManifestService.toAddJournalEntry(manifest, false);
          }

          return ManifestService.toAddJournalEntry(file, true);
        }),
      );
  }

  async #calculateChunkHash(sharePath: Buffer, manifest: FileManifest): Promise<FileManifest> {
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
      this.#logger.warn(`Can't read hash of the file ${sharePath.toString()}/${manifest.path.toString()}`);
      return manifest;
    }
  }
}
