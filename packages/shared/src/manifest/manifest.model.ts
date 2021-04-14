import { Logger } from '@nestjs/common';
import { rm, rename } from 'fs/promises';
import { join } from 'path';
import { concat, defer, EMPTY, Observable, from } from 'rxjs';
import { catchError, finalize, map, mergeMap, reduce } from 'rxjs/operators';

import { ProtoFileManifest, ProtoFileManifestJournalEntry } from '../manifest/object-proto.model';
import { EntryType, FileManifest, FileManifestJournalEntry } from '../models/manifest.model';
import { notUndefined, silence } from '../utils/observable.utils';
import { IndexManifest } from './index-manifest.model';
import { readAllMessages, writeAllMessages } from './manifest-wrapper.utils';

export class Manifest {
  private logger = new Logger(Manifest.name);

  private journalPath: string;
  private manifestPath: string;
  private newPath: string;
  private lockPath: string;

  constructor(manifestName: string, private path: string) {
    this.journalPath = join(path, `${manifestName}.journal`);
    this.manifestPath = join(path, `${manifestName}.manifest`);
    this.newPath = join(path, `${manifestName}.new`);
    this.lockPath = join(path, `${manifestName}.lock`);
  }

  static toRemoveJournalEntry(path: Buffer): FileManifestJournalEntry {
    return {
      type: EntryType.REMOVE,
      path,
    };
  }

  static toAddJournalEntry(manifest: FileManifest, add: boolean): FileManifestJournalEntry {
    return {
      type: add ? EntryType.ADD : EntryType.MODIFY,
      manifest,
    };
  }

  loadIndex(): Observable<IndexManifest> {
    const manifestWrapper = readAllMessages<FileManifest>(this.manifestPath, ProtoFileManifest).pipe(
      map(
        (frame) =>
          ({
            type: EntryType.ADD,
            manifest: frame.message,
          } as FileManifestJournalEntry),
      ),
      catchError((err) => {
        this.logger.warn(`Can't read the file ${this.manifestPath}: ${err.message}`);
        return EMPTY;
      }),
    );

    const journalWrapper = readAllMessages<FileManifestJournalEntry>(
      this.journalPath,
      ProtoFileManifestJournalEntry,
    ).pipe(
      map((frame) => frame.message),
      catchError((err) => {
        this.logger.warn(`Can't read the file ${this.journalPath}: ${err.message}`);
        return EMPTY;
      }),
    );

    const indexWrapper = concat(manifestWrapper, journalWrapper).pipe(
      reduce<FileManifestJournalEntry, IndexManifest>((index, journalEntry) => {
        if (journalEntry.type === EntryType.CLOSE) {
          return index;
        }

        index.process(journalEntry);

        return index;
      }, new IndexManifest()),
    );

    return indexWrapper;
  }

  writeJournalEntry(): (source: Observable<FileManifestJournalEntry>) => Observable<FileManifestJournalEntry> {
    return writeAllMessages<FileManifestJournalEntry>(this.journalPath, ProtoFileManifestJournalEntry);
  }

  async deleteManifest(): Promise<void> {
    await rm(this.newPath, { force: true });
    await rm(this.journalPath, { force: true });
    await rm(this.manifestPath, { force: true });
    await rm(this.lockPath, { force: true });
  }

  compact(): Observable<FileManifest> {
    const writeToManifest$ = this.loadIndex().pipe(
      mergeMap((index) => index.walk()),
      map((entry) => entry.manifest),
      notUndefined(),
      writeAllMessages(this.newPath, ProtoFileManifest),
    );

    const cleanupManifest$ = defer(async () => {
      await Promise.all([rm(this.journalPath, { force: true }), rm(this.manifestPath, { force: true })]);
      await rename(this.newPath, this.manifestPath);
    }).pipe(silence);

    return concat(writeToManifest$, cleanupManifest$);
  }
}
