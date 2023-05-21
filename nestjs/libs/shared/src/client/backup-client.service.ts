import { BadRequestException, Injectable, Logger } from '@nestjs/common';
import { ExecuteCommandService, globStringToRegex, mangle, notUndefined, split } from '@woodstock/core';
import { createReadStream, ReadStream } from 'fs';
import { AsyncIterableX, concat, from, of } from 'ix/asynciterable';
import { catchError, concatAll, filter, finalize, map } from 'ix/asynciterable/operators';
import { Manifest, ManifestService } from '../manifest';
import {
  ChunkInformation,
  ExecuteCommandReply,
  FileChunk,
  FileManifest,
  LaunchBackupReply,
  RefreshCacheReply,
  RefreshCacheRequest,
  Share,
  StatusCode,
} from '../protobuf/woodstock.interface';
import { FileHashReader, FileReaderService } from '../scanner';

@Injectable()
export class BackupOnClientService {
  private logger = new Logger(BackupOnClientService.name);

  constructor(
    private commandService: ExecuteCommandService,
    private fileReader: FileReaderService,
    private manifestService: ManifestService,
  ) { }

  async executeCommand(command: string): Promise<ExecuteCommandReply> {
    return this.commandService.executeCommand(command);
  }

  private async createManifestForSharePath(sharePath: Buffer, source: AsyncIterable<FileManifest | undefined>) {
    const manifest = new Manifest(`backups.${mangle(sharePath)}`, '/tmp/');
    await this.manifestService.deleteManifest(manifest);

    const entries$ = from(source).pipe(
      notUndefined<FileManifest>(),
      map((fileManifest) => ManifestService.toAddJournalEntry(fileManifest, true)),
    );
    await this.manifestService.writeJournalEntry(entries$, manifest);

    await this.manifestService.compact(manifest);
  }

  private async createManifestsFromSource(request: AsyncIterableX<RefreshCacheRequest>) {
    this.logger.log(`Start creating manifests from source`);

    const isHeader = (v: RefreshCacheRequest) => v.header?.sharePath;
    const groupOfManifest = request.pipe(split(isHeader));

    for await (const group of groupOfManifest) {
      const sharePath = group.key;
      const manifests = group.iterable.pipe(
        map((v) => v.fileManifest),
        notUndefined(),
      );

      await this.createManifestForSharePath(sharePath, manifests);
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
      map((entry) => ({ entry, response: undefined })),
    );

    const addRemoveToIndex$ = createIndex$.pipe(
      map((index) => from(index.walk())),
      concatAll(),
      filter((entry) => !entry.markViewed),
      map((file) => ManifestService.toRemoveJournalEntry(file.path)),
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
    this.logger.debug(`Get the chunk ${filename} at position ${position} (size: ${size})`);

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
