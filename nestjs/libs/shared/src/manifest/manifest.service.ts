import { Injectable, Logger } from '@nestjs/common';
import { isExists, notUndefined, ProtobufService } from '@woodstock/core';
import { rename } from 'fs/promises';
import { AsyncIterableX, concat, from, reduce } from 'ix/asynciterable';
import { catchError, concatAll, concatMap, filter, map } from 'ix/asynciterable/operators';
import { EntryType, FileManifest, FileManifestJournalEntry, PoolRefCount } from '../protobuf/woodstock.interface.js';
import { ProtoFileManifest, ProtoFileManifestJournalEntry } from '../protobuf/woodstock.model.js';
import { IndexManifest } from './index-manifest.model.js';
import { ManifestChunk } from './manifest.interface.js';
import { Manifest } from './manifest.model.js';

@Injectable()
export class ManifestService {
  private logger = new Logger(ManifestService.name);

  constructor(private readonly protobufService: ProtobufService) {}

  static toRemoveJournalEntry(path: Buffer): FileManifestJournalEntry {
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

  static toAddJournalEntry(manifest: FileManifest, add: boolean): FileManifestJournalEntry {
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
    const mappedSource = from(source).pipe(map(mapping), notUndefined());
    await this.protobufService.atomicWriteFile<FileManifestJournalEntry>(
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
    const mappedSource = from(source).pipe(map(mapping), notUndefined());
    await this.protobufService.atomicWriteFile<FileManifestJournalEntry>(
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
    return from(this.protobufService.loadFile<FileManifest>(manifest.manifestPath, ProtoFileManifest)).pipe(
      map((frame) => frame.message),
      catchError((err) => {
        this.logger.warn(`Can't read the file ${manifest.manifestPath}: ${err.message}`);
        return from([]);
      }),
    );
  }

  readJournalEntries(manifest: Manifest): AsyncIterableX<FileManifestJournalEntry> {
    try {
      return from(
        this.protobufService.loadFile<FileManifestJournalEntry>(manifest.journalPath, ProtoFileManifestJournalEntry),
      ).pipe(
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
    return from(
      this.protobufService.loadFile<FileManifestJournalEntry>(manifest.fileListPath, ProtoFileManifestJournalEntry),
    ).pipe(
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
    await this.protobufService.rmFile(manifest.fileListPath);
    await this.protobufService.rmFile(manifest.newPath);
    await this.protobufService.rmFile(manifest.journalPath);
    await this.protobufService.rmFile(manifest.manifestPath);
    await this.protobufService.rmFile(manifest.lockPath);
  }

  compact(manifest: Manifest, mapping?: (v: FileManifest) => Promise<FileManifest | undefined>): Promise<void>;
  compact<T>(manifest: Manifest, mapping: (v: T) => Promise<FileManifest | undefined>): Promise<void>;
  async compact<T>(
    manifest: Manifest,
    mapping: (v: T) => Promise<FileManifest | undefined> = (v) => v as any,
  ): Promise<void> {
    this.logger.debug(`Compact manifest from ${manifest.manifestPath}`);
    const allMessages$ = this.readAllMessages(manifest);

    await this.protobufService
      .writeFile(manifest.newPath, ProtoFileManifest, allMessages$.pipe(map(mapping)))
      .then(async () => {
        try {
          await Promise.all([
            this.protobufService.rmFile(manifest.journalPath),
            this.protobufService.rmFile(manifest.fileListPath),
            this.protobufService.rmFile(manifest.manifestPath),
          ]);
        } catch (err) {
        } finally {
          await rename(manifest.newPath, manifest.manifestPath);
          this.logger.debug(`[END] Compact manifest from ${manifest.manifestPath}`);
        }
      });
  }

  listChunksFromManifest(manifest: Manifest): AsyncIterableX<ManifestChunk> {
    return this.readManifestEntries(manifest).pipe(
      concatMap((manifest) => {
        const chunks = manifest.chunks ?? [];
        return from(chunks).pipe(map((sha256) => ({ sha256, manifest })));
      }),
      filter((chunk) => !!chunk.sha256),
    );
  }

  generateRefcntFromManifest(manifest: Manifest): AsyncIterableX<PoolRefCount> {
    return this.listChunksFromManifest(manifest).pipe(
      map((chunk) => ({
        sha256: chunk.sha256,
        refCount: 1,
        size: 0,
        compressedSize: 0,
      })),
    );
  }
}
