import { Injectable, Logger, LoggerService } from '@nestjs/common';
import {
  ApplicationConfigService,
  BackupsService,
  PoolChunkInformation,
  PoolChunkRefCnt,
  PoolService,
} from '@woodstock/backoffice-shared';
import {
  bigIntToLong,
  ChunkInformation,
  concurrentMap,
  EntryType,
  FileBrowserService,
  FileManifest,
  FileManifestJournalEntry,
  joinBuffer,
  LogLevel,
  longToBigInt,
  LONG_CHUNK_SIZE,
  ManifestService,
  ReferenceCount,
  RefreshCacheReply,
  RefreshCacheRequest,
  Share,
  StatusCode,
} from '@woodstock/shared';
import {
  AsyncSink,
  concat as concatIx,
  from as fromIx,
  of as ofIx,
  range as rangeIx,
  reduce as reduceIx,
} from 'ix/asynciterable';
import { concatAll, finalize, map as mapIx } from 'ix/asynciterable/operators';
import * as Long from 'long';
import { Observable } from 'rxjs';
import { Writable } from 'stream';
import { BackupClientGrpc, BackupsGrpcContext } from './backup-client-grpc.class';
import { LaunchBackupError } from './backup.error';

export const SHA256_EMPTYSTRING = Buffer.from(
  'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
  'hex',
);

@Injectable()
export class BackupClient {
  private logger = new Logger(BackupClient.name);

  constructor(
    private applicationConfig: ApplicationConfigService,
    private clientGrpc: BackupClientGrpc,
    private backupService: BackupsService,
    private manifestService: ManifestService,
    private poolService: PoolService,
    private poolChunkRefCnt: PoolChunkRefCnt,
  ) {}

  async authenticate(context: BackupsGrpcContext, logger: LoggerService): Promise<void> {
    this.logger.log(`Authenticate to ${context.host} (${context.ip})`);

    const reply = await this.clientGrpc.authenticate(context);

    if (!reply || reply.code === StatusCode.Failed || !reply.sessionId) {
      throw new LaunchBackupError('Authentication failed');
    }
    context.logger = logger;

    const ac = new AbortController();
    context.abortable.push(ac);

    const streamLog = this.clientGrpc.streamLog(context);
    streamLog
      .forEach(
        (log) => {
          switch (log.level) {
            case LogLevel.verbose:
              logger.debug?.(log.line, log.context);
              break;
            case LogLevel.debug:
              logger.debug?.(log.line, log.context);
              break;
            case LogLevel.error:
              logger.error(log.line, log.context);
              break;
            case LogLevel.warn:
              logger.warn(log.line, log.context);
              break;
            default:
              logger.log(log.line, log.context);
              break;
          }
        },
        { signal: ac.signal },
      )
      .then(() => {
        logger.log('The backup process has been completed', 'logger');
      })
      .catch((err) => {
        logger.error(`The backup process has been failed: ${err.message}`, 'logger');
      });

    this.logger.log(`Authentication has been completed: ${context.sessionId}`);
  }

  executeCommand(context: BackupsGrpcContext, command: string): Promise<void> {
    this.logger.log(`Execute command (${context.sessionId}): ${command}`);

    context.logger?.log(`Execute command: ${command}`, 'executeCommand');

    return this.clientGrpc.executeCommand(context, command);
  }

  getFileList(context: BackupsGrpcContext, backupShare: Share): Observable<FileManifestJournalEntry> {
    this.logger.log(`Get file list (${context.sessionId}): ${backupShare.sharePath.toString()}`);
    const manifest = this.backupService.getManifest(context.host, context.currentBackupId, backupShare.sharePath);

    return new Observable<FileManifestJournalEntry>((subscriber) => {
      const launchBackup = this.clientGrpc.downloadFileList(context, backupShare);
      this.manifestService
        .writeFileListEntry(launchBackup, manifest, async (entry) => {
          if (entry) {
            subscriber.next(entry);
          }
          return entry;
        })
        .then(() => {
          subscriber.complete();
        })
        .catch((err) => {
          subscriber.error(err);
        });

      return () => {
        // FIXME: Abort the stream
      };
    });
  }

  private async copyChunk(
    context: BackupsGrpcContext,
    sharePath: Buffer,
    fileManifest: FileManifest,
    chunkNumber: number,
  ): Promise<PoolChunkInformation> {
    const sha256 = chunkNumber < (fileManifest.chunks?.length || 0) ? fileManifest.chunks[chunkNumber] : undefined;
    const wrapper = this.poolService.getChunk(sha256);

    const position = LONG_CHUNK_SIZE.mul(chunkNumber);
    const restSize = (fileManifest.stats?.size || Long.ZERO).sub(position);
    const size = LONG_CHUNK_SIZE.gt(restSize) ? restSize : LONG_CHUNK_SIZE;

    try {
      if (size.greaterThan(0)) {
        const chunk: ChunkInformation = {
          filename: joinBuffer(sharePath, fileManifest.path),
          position,
          size,
          sha256,
        };
        // this.logger.log(`Get the chunk ${fileManifest.path}:${position}-${chunk.size.toNumber()}`);

        // FIXME: this.logger.error(`${fileManifest.path.toString()}:${chunkNumber}: ${(err as Error).message}`, err);
        if (await wrapper.exists()) {
          // Read the chunk
          const oldChunk = await wrapper.read(
            new Writable({
              write(_, _2, callback) {
                setImmediate(callback);
              },
            }),
          );

          if (!sha256 || !oldChunk.sha256.equals(sha256)) {
            this.logger.error(
              `${fileManifest.path.toString()}:${chunkNumber}: Chunk ${sha256} is not the same that ${
                oldChunk.sha256
              }.`,
            );
          }

          return oldChunk;
        } else {
          // Create the chunk
          const readable = this.clientGrpc.copyChunk(context, chunk);
          return await wrapper.write(readable);
        }
      }

      return {
        size: 0n,
        compressedSize: 0n,
        sha256: SHA256_EMPTYSTRING,
      };
    } catch (err) {
      this.logger.error(`${fileManifest.path.toString()}:${chunkNumber}: ${(err as Error).message}`, err);
      throw err;
    }
  }

  private async downloadManifestFile(
    context: BackupsGrpcContext,
    sharePath: Buffer,
    fileManifest: FileManifest,
    chunks: AsyncSink<PoolChunkInformation>,
  ): Promise<FileManifest> {
    let chunkLength = (fileManifest.stats?.size || Long.ZERO).div(LONG_CHUNK_SIZE).toNumber();
    if ((fileManifest.stats?.size || Long.ZERO).mod(LONG_CHUNK_SIZE).greaterThan(Long.ZERO)) {
      chunkLength++;
    }

    const chunksIt = rangeIx(0, chunkLength).pipe(
      mapIx(async (chunkNumber) => {
        const chunk = await this.copyChunk(context, sharePath, fileManifest, chunkNumber);
        return { chunkNumber, chunk };
      }),
    );

    const manifestSize = await reduceIx(chunksIt, {
      seed: { compressedSize: 0n, size: 0n, chunks: [] as Buffer[] },
      callback: (acc, { chunkNumber, chunk }) => {
        acc.compressedSize += chunk.compressedSize;
        acc.size += chunk.size;
        acc.chunks[chunkNumber] = chunk.sha256;
        chunks.write(chunk);
        return acc;
      },
    });

    fileManifest.stats = fileManifest.stats || {};
    fileManifest.stats.compressedSize = bigIntToLong(manifestSize.compressedSize);
    if (!bigIntToLong(manifestSize.size).equals(fileManifest.stats.size || Long.ZERO)) {
      this.logger.error(
        `The manifest of file ${fileManifest.path.toString()} size (${fileManifest.stats.size?.toString()}) is not equal to the sum of chunk size ${manifestSize.size.toString()}`,
      );
      fileManifest.stats.size = bigIntToLong(manifestSize.size);
    }
    fileManifest.chunks = manifestSize.chunks;

    return fileManifest;
  }

  createBackup(
    context: BackupsGrpcContext,
    backupShare: Share,
  ): Observable<FileManifestJournalEntry | PoolChunkInformation> {
    this.logger.log(`Create backup (${context.sessionId}): ${backupShare.sharePath.toString()}`);
    const manifest = this.backupService.getManifest(context.host, context.currentBackupId, backupShare.sharePath);

    return new Observable<FileManifestJournalEntry | PoolChunkInformation>((subscriber) => {
      const chunkSink = new AsyncSink<PoolChunkInformation>();
      const entries = this.manifestService.readFilelistEntries(manifest).pipe(
        // FIXME: Define the concurrency
        concurrentMap<FileManifestJournalEntry, FileManifestJournalEntry | undefined>(20, async (entry) => {
          try {
            if (
              entry?.type !== EntryType.REMOVE &&
              entry?.manifest &&
              !FileBrowserService.isSpecialFile(longToBigInt(entry?.manifest?.stats?.mode || Long.ZERO))
            ) {
              const manifest = await this.downloadManifestFile(
                context,
                backupShare.sharePath,
                entry.manifest,
                chunkSink,
              );
              return { type: entry.type, manifest };
            } else {
              return entry;
            }
          } catch (err) {
            // FIXME: Gérer l'erreur
            console.log(err.stack);
            this.logger.error(`${entry.manifest?.path.toString()}: ${(err as Error).message}`, err);
            return undefined;
          }
        }),
        finalize(() => chunkSink.end()),
      );

      const journalEntry = this.manifestService.writeJournalEntry(entries, manifest, async (entry) => {
        if (entry) {
          subscriber.next(entry);
        }
        return entry;
      });

      const refCntEntry = this.poolChunkRefCnt.writeJournal(
        chunkSink,
        this.backupService.getDestinationDirectory(context.host, context.currentBackupId),
        async (entry) => {
          if (entry) {
            subscriber.next(entry);
          }
          return {
            sha256: entry.sha256,
            refCount: 1,
            size: Number(entry.size),
            compressedSize: Number(entry.compressedSize),
          };
        },
      );

      Promise.all([journalEntry, refCntEntry])
        .then(() => {
          subscriber.complete();
        })
        .catch((err) => {
          subscriber.error(err);
        });

      return () => {
        // FIXME: Abort the stream
      };
    });
  }

  compact(context: BackupsGrpcContext, sharePath: Buffer): Observable<FileManifest> {
    this.logger.log('Counting reference for the share: ' + sharePath.toString());
    const manifest = this.backupService.getManifest(context.host, context.currentBackupId, sharePath);

    return new Observable<FileManifest>((subscriber) => {
      const compactManifest = this.manifestService.compact(manifest, async (v) => {
        if (v) {
          subscriber.next(v);
          return v;
        }
        return undefined;
      });

      compactManifest
        .then(() => {
          subscriber.complete();
        })
        .catch((err) => {
          subscriber.error(err);
        });
    });
  }

  async countRef(context: BackupsGrpcContext): Promise<void> {
    this.logger.log('Counting reference');
    const hostDirectory = this.backupService.getHostDirectory(context.host);
    const destinationDirectory = this.backupService.getDestinationDirectory(context.host, context.currentBackupId);

    const refcnt = new ReferenceCount(hostDirectory, destinationDirectory, this.applicationConfig.poolPath);

    await this.poolChunkRefCnt.compact(refcnt);
  }

  refreshCache(context: BackupsGrpcContext, shares: string[]): Promise<RefreshCacheReply> {
    const shares$ = fromIx(shares).pipe(mapIx((s) => Buffer.from(s)));
    const request$ = shares$.pipe(
      mapIx((share) => {
        const manifest = this.backupService.getManifest(context.host, context.currentBackupId, share);
        return concatIx(
          ofIx({ header: { sharePath: share }, fileManifest: undefined } as RefreshCacheRequest),
          this.manifestService
            .readManifestEntries(manifest)
            .pipe(mapIx((fileManifest) => ({ header: undefined, fileManifest } as RefreshCacheRequest))),
        );
      }),
      concatAll<RefreshCacheRequest>(),
    );

    return this.clientGrpc.refreshCache(context, request$);
  }

  close(context: BackupsGrpcContext): void {
    this.logger.log(`Close connection (${context.sessionId})`);
    try {
      // Stop subscription
      context.abortable.forEach((s) => s.abort());
      context.abortable = [];
      context.logger = undefined;

      // Close connection
      context.sessionId = undefined;
      context.client.close();

      this.logger.log(`Connection closed (${context.sessionId})`);
    } catch (err) {
      this.logger.error(`Error closing connection (${context.sessionId})`, err);
    }
  }
}
