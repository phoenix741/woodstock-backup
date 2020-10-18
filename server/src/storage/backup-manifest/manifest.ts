import { Logger } from '@nestjs/common';
import { promises } from 'fs';
import * as mkdirp from 'mkdirp';
import { join } from 'path';
import { concat, EMPTY, Observable } from 'rxjs';
import { catchError, finalize, map, mergeMap, reduce } from 'rxjs/operators';

import { notUndefined } from '../../utils/lodash';
import { IndexManifest } from './index-manifest.model';
import { readAllMessages, writeAllMessages } from './manifest-wrapper.utils';
import { EntryType, FileManifest, FileManifestJournalEntry } from './manifest.model';
import { ProtoFileManifest, ProtoFileManifestJournalEntry } from './proto.utils';

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

    mkdirp(this.path);
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

        if (journalEntry.type === EntryType.REMOVE) {
          index.remove(journalEntry.path);
        } else {
          const entry = index.add(journalEntry.manifest.path);
          entry.manifest = journalEntry.manifest;
        }
        return index;
      }, new IndexManifest()),
    );

    return indexWrapper;
  }

  writeJournalEntry(): (source: Observable<FileManifestJournalEntry>) => Observable<FileManifestJournalEntry> {
    return writeAllMessages<FileManifestJournalEntry>(this.journalPath, ProtoFileManifestJournalEntry);
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

  async deleteManifest(): Promise<void> {
    await promises.rm(this.newPath, { force: true });
    await promises.rm(this.journalPath, { force: true });
    await promises.rm(this.manifestPath, { force: true });
    await promises.rm(this.lockPath, { force: true });
  }

  compact(): Observable<FileManifest> {
    return this.loadIndex().pipe(
      mergeMap((index) => index.walk()),
      map((entry) => entry.manifest),
      notUndefined(),
      writeAllMessages(this.newPath, ProtoFileManifest),
      finalize(async () => {
        await Promise.all([
          promises.rm(this.journalPath, { force: true }),
          promises.rm(this.manifestPath, { force: true }),
        ]);
        await promises.rename(this.newPath, this.manifestPath);
      }),
    );
  }
}
