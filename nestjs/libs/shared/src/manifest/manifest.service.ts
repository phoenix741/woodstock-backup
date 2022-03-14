import { Injectable, Logger } from '@nestjs/common';
import { constants as constantsFs } from 'fs';
import { access, rename, unlink } from 'fs/promises';
import { AsyncIterableX, concat, from, pipe, reduce } from 'ix/asynciterable';
import { catchError, concatAll, map } from 'ix/asynciterable/operators';
import { ProtoFileManifest, ProtoFileManifestJournalEntry } from '../models/object-proto.model';
import { EntryType, FileManifest, FileManifestJournalEntry } from '../models/woodstock';
import { ProtobufService } from '../services/protobuf.service';
import { notUndefined } from '../utils/iterator.utils';
import { IndexManifest } from './index-manifest.model';
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

  constructor(private readonly protobufService: ProtobufService) {}

  toRemoveJournalEntry(path: Buffer): FileManifestJournalEntry {
    return {
      type: EntryType.REMOVE,
      manifest: {
        path,
        stats: undefined,
        xattr: {},
        acl: [],
        chunks: [],
      },
    };
  }

  toAddJournalEntry(manifest: FileManifest, add: boolean): FileManifestJournalEntry {
    return {
      type: add ? EntryType.ADD : EntryType.MODIFY,
      manifest,
    };
  }

  async writeJournalEntry(
    source: AsyncIterable<FileManifestJournalEntry>,
    manifest: Manifest,
    mapping?: (v: FileManifestJournalEntry) => Promise<FileManifestJournalEntry | undefined>,
  ): Promise<void>;
  async writeJournalEntry<T = FileManifestJournalEntry>(
    source: AsyncIterable<T>,
    manifest: Manifest,
    mapping: (v: T) => Promise<FileManifestJournalEntry | undefined>,
  ): Promise<void>;
  async writeJournalEntry<T = FileManifestJournalEntry>(
    source: AsyncIterable<T>,
    manifest: Manifest,
    mapping: (v: T) => Promise<FileManifestJournalEntry | undefined> = (v) => v as any,
  ): Promise<void> {
    const mappedSource = pipe(source, map(mapping), notUndefined());
    return await this.protobufService.writeFile<FileManifestJournalEntry>(
      manifest.journalPath,
      ProtoFileManifestJournalEntry,
      mappedSource,
    );
  }

  async writeFileListEntry(
    source: AsyncIterable<FileManifestJournalEntry>,
    manifest: Manifest,
    mapping?: (v: FileManifestJournalEntry) => Promise<FileManifestJournalEntry | undefined>,
  ): Promise<void>;
  async writeFileListEntry<T = FileManifestJournalEntry>(
    source: AsyncIterable<T>,
    manifest: Manifest,
    mapping: (v: T) => Promise<FileManifestJournalEntry | undefined>,
  ): Promise<void>;
  async writeFileListEntry<T = FileManifestJournalEntry>(
    source: AsyncIterable<T>,
    manifest: Manifest,
    mapping: (v: T) => Promise<FileManifestJournalEntry | undefined> = (v) => v as any,
  ): Promise<void> {
    const mappedSource = pipe(source, map(mapping), notUndefined());
    return await this.protobufService.writeFile<FileManifestJournalEntry>(
      manifest.fileListPath,
      ProtoFileManifestJournalEntry,
      mappedSource,
    );
  }

  readAllMessages(manifest: Manifest): AsyncIterableX<FileManifest> {
    return from(this.loadIndex(manifest)).pipe(
      map((index) => from(index.walk())),
      concatAll(),
      map((entry) => entry.manifest),
      notUndefined(),
    );
  }

  readManifestEntries(manifest: Manifest): AsyncIterableX<FileManifest> {
    return pipe(
      this.protobufService.loadFile<FileManifest>(manifest.manifestPath, ProtoFileManifest),
      map((frame) => frame.message),
      catchError((err) => {
        this.logger.warn(`Can't read the file ${manifest.manifestPath}: ${err.message}`);
        return from([]);
      }),
    );
  }

  readJournalEntries(manifest: Manifest): AsyncIterableX<FileManifestJournalEntry> {
    try {
      return pipe(
        this.protobufService.loadFile<FileManifestJournalEntry>(manifest.journalPath, ProtoFileManifestJournalEntry),
        map((frame) => frame.message),
        catchError((err) => {
          this.logger.warn(`Can't read the file ${manifest.journalPath}: ${err.message}`);
          return from([]);
        }),
      );
    } catch (err) {
      return from([]);
    }
  }

  readFilelistEntries(manifest: Manifest): AsyncIterableX<FileManifestJournalEntry> {
    return pipe(
      this.protobufService.loadFile<FileManifestJournalEntry>(manifest.fileListPath, ProtoFileManifestJournalEntry),
      map((frame) => frame.message),
      catchError((err) => {
        this.logger.warn(`Can't read the file ${manifest.journalPath}: ${err.message}`);
        return from([]);
      }),
    );
  }

  async loadIndex(manifest: Manifest): Promise<IndexManifest> {
    const manifestWrapper = this.readManifestEntries(manifest).pipe(
      map(
        (manifest) =>
          ({
            type: EntryType.ADD,
            manifest,
          } as FileManifestJournalEntry),
      ),
    );
    const journalWrapper = this.readJournalEntries(manifest);

    const indexWrapper = reduce<FileManifestJournalEntry, IndexManifest>(concat(manifestWrapper, journalWrapper), {
      callback: (index, journalEntry) => {
        index.process(journalEntry);

        return index;
      },
      seed: new IndexManifest(),
    });
    return indexWrapper;
  }

  async exists(manifest: Manifest): Promise<boolean> {
    return (await isExists(manifest.manifestPath)) && !(await isExists(manifest.journalPath));
  }

  async deleteManifest(manifest: Manifest): Promise<void> {
    await rm(manifest.fileListPath);
    await rm(manifest.newPath);
    await rm(manifest.journalPath);
    await rm(manifest.manifestPath);
    await rm(manifest.lockPath);
  }

  compact(manifest: Manifest, mapping?: (v: FileManifest) => Promise<FileManifest | undefined>): Promise<void>;
  compact<T>(manifest: Manifest, mapping: (v: T) => Promise<FileManifest | undefined>): Promise<void>;
  async compact<T>(
    manifest: Manifest,
    mapping: (v: T) => Promise<FileManifest | undefined> = (v) => v as any,
  ): Promise<void> {
    this.logger.log(`Compact manifest from ${manifest.manifestPath}`);
    const allMessages$ = this.readAllMessages(manifest);

    await this.protobufService
      .writeFile(manifest.newPath, ProtoFileManifest, allMessages$.pipe(map(mapping)))
      .then(async () => {
        try {
          await Promise.all([rm(manifest.journalPath), rm(manifest.fileListPath), rm(manifest.manifestPath)]);
        } catch (err) {
        } finally {
          await rename(manifest.newPath, manifest.manifestPath);
          this.logger.log(`[END] Compact manifest from ${manifest.manifestPath}`);
        }
      });
  }
}
