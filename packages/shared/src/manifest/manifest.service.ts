import { Injectable, Logger } from '@nestjs/common';
import { constants as constantsFs } from 'fs';
import { access, rename, unlink } from 'fs/promises';
import { concat, defer, EMPTY, Observable } from 'rxjs';
import { catchError, map, mergeMap, reduce, tap, finalize } from 'rxjs/operators';

import { ProtoFileManifest, ProtoFileManifestJournalEntry } from '../manifest/object-proto.model';
import { EntryType, FileManifest, FileManifestJournalEntry } from '../models/manifest.model';
import { notUndefined, silence } from '../utils/observable.utils';
import { IndexManifest } from './index-manifest.model';
import { readAllMessages, writeAllMessages } from './manifest-wrapper.utils';
import { Manifest } from './manifest.model';

async function rm(path: string) {
  try {
    await unlink(path);
  } catch (err) {}
}

async function isExists(path: string) {
  return access(path, constantsFs.F_OK)
    .then(() => true)
    .catch(() => false);
}

@Injectable()
export class ManifestService {
  private logger = new Logger(ManifestService.name);

  toRemoveJournalEntry(path: Buffer): FileManifestJournalEntry {
    return {
      type: EntryType.REMOVE,
      manifest: {
        path,
      },
    };
  }

  toAddJournalEntry(manifest: FileManifest, add: boolean): FileManifestJournalEntry {
    return {
      type: add ? EntryType.ADD : EntryType.MODIFY,
      manifest,
    };
  }

  writeJournalEntry<T = FileManifestJournalEntry>(
    manifest: () => Manifest,
    mapping: (v: T) => FileManifestJournalEntry | undefined = (v) => v as any,
  ): (source: Observable<T>) => Observable<T> {
    return writeAllMessages<T, FileManifestJournalEntry>(
      () => manifest().journalPath,
      mapping,
      ProtoFileManifestJournalEntry,
    );
  }

  loadIndex(manifest: Manifest): Observable<IndexManifest> {
    const manifestWrapper = readAllMessages<FileManifest>(manifest.manifestPath, ProtoFileManifest).pipe(
      map(
        (frame) =>
          ({
            type: EntryType.ADD,
            manifest: frame.message,
          } as FileManifestJournalEntry),
      ),
      catchError((err) => {
        this.logger.warn(`Can't read the file ${manifest.manifestPath}: ${err.message}`);
        return EMPTY;
      }),
    );

    const journalWrapper = readAllMessages<FileManifestJournalEntry>(
      manifest.journalPath,
      ProtoFileManifestJournalEntry,
    ).pipe(
      map((frame) => frame.message),
      catchError((err) => {
        this.logger.warn(`Can't read the file ${manifest.journalPath}: ${err.message}`);
        return EMPTY;
      }),
    );

    const indexWrapper = concat(manifestWrapper, journalWrapper).pipe(
      reduce<FileManifestJournalEntry, IndexManifest>((index, journalEntry) => {
        index.process(journalEntry);

        return index;
      }, new IndexManifest()),
    );

    return indexWrapper;
  }

  async exists(manifest: Manifest) {
    return isExists(manifest.manifestPath) && !isExists(manifest.journalPath);
  }

  async deleteManifest(manifest: Manifest): Promise<void> {
    await rm(manifest.newPath);
    await rm(manifest.journalPath);
    await rm(manifest.manifestPath);
    await rm(manifest.lockPath);
  }

  compact(manifest: Manifest): Observable<FileManifest> {
    const writeToManifest$ = this.loadIndex(manifest).pipe(
      mergeMap((index) => index.walk()),
      map((entry) => entry.manifest),
      notUndefined(),
      writeAllMessages(
        () => manifest.newPath,
        (m) => m,
        ProtoFileManifest,
      ),
    );

    const cleanupManifest$ = defer(async () => {
      await Promise.all([rm(manifest.journalPath), rm(manifest.manifestPath)]);
      await rename(manifest.newPath, manifest.manifestPath);
    }).pipe(silence);

    return concat(writeToManifest$, cleanupManifest$);
  }
}
