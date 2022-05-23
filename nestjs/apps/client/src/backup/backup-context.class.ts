import { BadRequestException, InternalServerErrorException, Logger } from '@nestjs/common';
import {
  ChunkInformation,
  FileChunk,
  FileHashReader,
  FileManifest,
  FileReader,
  globStringToRegex,
  LaunchBackupReply,
  mangle,
  Manifest,
  ManifestService,
  notUndefined,
  RefreshCacheReply,
  RefreshCacheRequest,
  Share,
  StatusCode
} from '@woodstock/shared';
import { createReadStream, ReadStream } from 'fs';
import { throwIfAborted } from 'ix/aborterror';
import { AsyncIterableX, concat, create, from, of, pipe } from 'ix/asynciterable';
import { catchError, concatAll, filter, finalize, map } from 'ix/asynciterable/operators';

export class BackupContext {
  private logger = new Logger(BackupContext.name);

  constructor(private fileReader: FileReader, private manifestService: ManifestService) {}

  private async createManifestForSharePath(sharePath: Buffer, source: AsyncIterable<FileManifest | undefined>) {
    const manifest = new Manifest(`backups.${mangle(sharePath)}`, '/tmp/');
    await this.manifestService.deleteManifest(manifest);

    const entries$ = pipe(
      source,
      notUndefined<FileManifest>(),
      map((fileManifest) => this.manifestService.toAddJournalEntry(fileManifest, true)),
    );
    await this.manifestService.writeJournalEntry(entries$, manifest);

    await this.manifestService.compact(manifest);
  }

  private async createManifestsFromSource(request: AsyncIterableX<RefreshCacheRequest>) {
    // First search the first header
    const it = request[Symbol.asyncIterator]();
    let next: IteratorResult<RefreshCacheRequest, RefreshCacheRequest>;
    let sharePath: Buffer | undefined;
    let firstManifest: FileManifest | undefined;

    while (!(next = await it.next()).done) {
      const { header, fileManifest } = next.value;
      sharePath = header?.sharePath;
      firstManifest = fileManifest;
      if (sharePath) {
        break;
      }
    }
    if (!sharePath) {
      throw new InternalServerErrorException();
    }

    while (!next.done) {
      // Then create an iterator for the rest
      const dataIt = create<FileManifest | undefined>((signal) => {
        return {
          async next() {
            throwIfAborted(signal);

            next = await it.next();
            if (next.done) {
              return { done: true, value: undefined };
            }
            const { header, fileManifest } = next.value;
            if (header?.sharePath) {
              sharePath = header.sharePath;
              firstManifest = fileManifest;
              return { done: true, value: undefined };
            }
            return { value: fileManifest };
          },
        };
      });

      // Then create the manifest
      await this.createManifestForSharePath(sharePath, firstManifest ? concat(of(firstManifest), dataIt) : dataIt);
    }
  }

  async refreshCache(request: AsyncIterableX<RefreshCacheRequest>): Promise<RefreshCacheReply> {
    try {
      await this.createManifestsFromSource(request);

      return { code: StatusCode.Ok };
    } catch (err) {
      return { code: StatusCode.Failed, message: err.message };
    }
  }

  launchBackup(share: Share): AsyncIterableX<LaunchBackupReply> {
    const manifest = new Manifest(`backups.${mangle(share.sharePath)}`, '/tmp/');

    const createIndex$ = from(this.manifestService.loadIndex(manifest));

    const addEntryToIndex$ = createIndex$.pipe(
      map((index) =>
        this.fileReader.getFiles(
          index,
          share.sharePath,
          share.includes?.map((s) => globStringToRegex(s.toString('latin1'))),
          share.excludes?.map((s) => globStringToRegex(s.toString('latin1'))),
        ),
      ),
      concatAll(),
      map((entry) => this.manifestService.toAddJournalEntry(entry, true)),
      map((entry) => ({ entry, response: undefined })),
    );

    const addRemoveToIndex$ = createIndex$.pipe(
      map((index) => from(index.walk())),
      concatAll(),
      filter((entry) => !entry.markViewed),
      map((file) => this.manifestService.toRemoveJournalEntry(file.path)),
      map((entry) => ({ entry, response: undefined })),
    );

    const entries$ = concat(
      addEntryToIndex$,
      addRemoveToIndex$,
      of({ entry: undefined, response: { code: StatusCode.Ok } }),
    );

    return entries$.pipe(
      catchError((err) => {
        return of({
          entry: undefined,
          response: { code: StatusCode.Failed, message: err.message },
        });
      }),
    );
  }

  getChunk(request: ChunkInformation & { failIfWrongHash?: boolean }): AsyncIterableX<FileChunk> {
    const { filename, position, size, sha256 } = request;
    const stream: ReadStream = createReadStream(filename, {
      start: position.toNumber(),
      end: position.add(size).sub(1).toNumber(),
    });
    const hashReader = new FileHashReader();

    stream.pipe(hashReader);
    stream.on('error', (err) => {
      hashReader.emit('error', err);
      hashReader.end();
      stream.close();
    });
    hashReader.on('error', () => {
      hashReader.end();
      stream.close();
    });

    return from(hashReader).pipe(
      map((data) => ({ data })),
      finalize(() => {
        if (request.failIfWrongHash && sha256 && hashReader.hash && !hashReader.hash.equals(sha256)) {
          const message = `The chunk ${filename.toString()}:${position}:${size} should have a sha of ${sha256.toString(
            'hex',
          )}, but is ${hashReader.hash.toString('hex')}`;
          this.logger.warn(message);

          throw new BadRequestException(message);
        }
      }),
    );
  }
}
