import { Injectable, Logger } from '@nestjs/common';
import {
  ApplicationConfigService,
  bigIntToLong,
  concurrentMap,
  joinBuffer,
  longToBigInt,
  LONG_CHUNK_SIZE,
} from '@woodstock/core';
import {
  BackupClientContext,
  BackupLogger,
  BackupsService,
  PoolChunkInformation,
  PoolService,
  RefCntService,
  ReferenceCount,
} from '@woodstock/server';
import {
  ChunkInformation,
  EntryType,
  ExecuteCommandReply,
  FileBrowserService,
  FileManifest,
  FileManifestJournalEntry,
  LogLevel,
  ManifestService,
  RefreshCacheReply,
  RefreshCacheRequest,
  Share,
  StatusCode,
} from '@woodstock/shared';
import {
  AsyncSink,
  concat as concatIx,
  from,
  from as fromIx,
  of as ofIx,
  range as rangeIx,
  reduce as reduceIx,
} from 'ix/asynciterable';
import { concatAll, concatMap, finalize, map, map as mapIx } from 'ix/asynciterable/operators';
import * as Long from 'long';
import { Observable } from 'rxjs';
import { BackupClientGrpc } from './backup-client-grpc.class.js';
import { BackupClientLocal } from './backup-client-local.class.js';
import { BackupClientInterface } from './backup-client.interface.js';
import { LaunchBackupError } from './backup.error.js';

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
    private clientLocal: BackupClientLocal,

    private backupService: BackupsService,
    private manifestService: ManifestService,
    private poolService: PoolService,
    private poolChunkRefCnt: RefCntService,
  ) {}

  #getClientInterface(context: BackupClientContext): BackupClientInterface {
    if (context.isLocal) {
      return this.clientLocal;
    }
    return this.clientGrpc;
  }

  createContext(
    ip: string | undefined,
    hostname: string,
    currentBackupId: number,
    pathPrefix?: string,
    originalDate?: number,
  ): BackupClientContext {
    if (pathPrefix) {
      return this.clientLocal.createContext(hostname, currentBackupId, pathPrefix, originalDate);
    }
    return this.clientGrpc.createContext(ip, hostname, currentBackupId, originalDate);
  }

  async createConnection(context: BackupClientContext): Promise<void> {
    await this.#getClientInterface(context).createConnection(context);
  }

  async authenticate(
    context: BackupClientContext,
    logger: BackupLogger,
    clientLogger: BackupLogger,
    password: string,
  ): Promise<void> {
    this.logger.log(`Authenticate to ${context.host} (${context.ip})`);

    const reply = await this.#getClientInterface(context).authenticate(context, password);

    if (!reply || reply.code === StatusCode.Failed || !reply.sessionId) {
      throw new LaunchBackupError('Authentication failed');
    }
    context.logger = logger;

    const ac = new AbortController();
    context.abortable.push(ac);

    const streamLog = this.#getClientInterface(context).streamLog(context);
    streamLog
      .forEach(
        (log) => {
          switch (log.level) {
            case LogLevel.verbose:
              clientLogger.debug?.(log.line, log.context);
              break;
            case LogLevel.debug:
              clientLogger.debug?.(log.line, log.context);
              break;
            case LogLevel.error:
              clientLogger.error(log.line, log.context);
              break;
            case LogLevel.warn:
              clientLogger.warn(log.line, log.context);
              break;
            default:
              clientLogger.log(log.line, log.context);
              break;
          }
        },
        { signal: ac.signal },
      )
      .then(() => {
        clientLogger.log('The backup process has been completed', 'logger');
      })
      .catch((err) => {
        clientLogger.error(`The backup process has been failed: ${err.message}`, 'logger');
      });

    this.logger.log(`Authentication has been completed: ${context.sessionId}`);
  }

  async executeCommand(context: BackupClientContext, command: string): Promise<ExecuteCommandReply> {
    this.logger.log(`Execute command (${context.sessionId}): ${command}`);

    context.logger?.log(`Execute command: ${command}`, 'executeCommand');

    return await this.#getClientInterface(context).executeCommand(context, command);
  }

  getFileList(context: BackupClientContext, backupShare: Share): Observable<FileManifestJournalEntry> {
    this.logger.log(`Get file list (${context.sessionId}): ${backupShare.sharePath.toString()}`);
    const manifest = this.backupService.getManifest(context.host, context.currentBackupId, backupShare.sharePath);

    return new Observable<FileManifestJournalEntry>((subscriber) => {
      const launchBackup = this.#getClientInterface(context).downloadFileList(context, backupShare);
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
    context: BackupClientContext,
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
          const oldChunk = await wrapper.getChunkInformation();

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
          const readable = this.#getClientInterface(context).copyChunk(context, chunk);
          return await wrapper.write(readable, joinBuffer(sharePath, fileManifest.path).toString());
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
    context: BackupClientContext,
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
    context: BackupClientContext,
    backupShare: Share,
    maxConcurrentDownloads = 10,
  ): Observable<FileManifestJournalEntry | PoolChunkInformation> {
    this.logger.log(`Create backup (${context.sessionId}): ${backupShare.sharePath.toString()}`);
    const manifest = this.backupService.getManifest(context.host, context.currentBackupId, backupShare.sharePath);

    return new Observable<FileManifestJournalEntry | PoolChunkInformation>((subscriber) => {
      let errorCount = 0;
      const chunkSink = new AsyncSink<PoolChunkInformation>();
      const entries = this.manifestService.readFilelistEntries(manifest).pipe(
        concurrentMap<FileManifestJournalEntry, FileManifestJournalEntry | Error>(
          maxConcurrentDownloads,
          async (entry) => {
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
              // FIXME: Gérer l'erreur, quelle erreur quand le fichier change ou est supprimé???
              errorCount++;
              this.logger.verbose(
                `Can't download chunk for ${entry.manifest?.path.toString()}: '${(err as Error).message}'`,
              );
              return err;
            }
          },
        ),
        finalize(() => {
          this.logger.verbose('All chunks are downloaded');
          chunkSink.end();
        }),
      );

      const journalEntry = this.manifestService.writeJournalEntry(entries, manifest, async (entry) => {
        const isError = entry instanceof Error;
        if (!isError) {
          subscriber.next(entry);
          return entry;
        }
        return undefined;
      });

      const referenceCountFiles = new ReferenceCount(
        this.backupService.getHostDirectory(context.host),
        this.backupService.getDestinationDirectory(context.host, context.currentBackupId),
        this.applicationConfig.poolPath,
      );
      const refCntEntry = this.poolChunkRefCnt.addChunkInformationToRefCnt(
        fromIx(chunkSink).pipe(
          map(async (entry) => {
            this.logger.verbose(
              `Add chunk to refcnt: ${entry.sha256.toString('hex')} (size = ${entry.size}, compressedSize = ${
                entry.compressedSize
              }))`,
            );
            if (entry) {
              subscriber.next(entry);
            }
            return {
              sha256: entry.sha256,
              refCount: 0,
              size: Number(entry.size),
              compressedSize: Number(entry.compressedSize),
            };
          }),
        ),
        referenceCountFiles.backupPath,
      );

      Promise.all([journalEntry, refCntEntry])
        .then(() => {
          if (errorCount === 0) {
            subscriber.complete();
          } else {
            subscriber.error(new Error(`Can't download ${errorCount} chunks`));
          }
        })
        .catch((err) => {
          subscriber.error(err);
        });

      return () => {
        // FIXME: Abort the stream
      };
    });
  }

  compact(context: BackupClientContext, sharePath: Buffer): Observable<FileManifest> {
    this.logger.log('Compact backup for the share: ' + sharePath.toString());
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
        .then(async () => {
          await this.backupService.addBackupSharePath(context.host, context.currentBackupId, sharePath);
          subscriber.complete();
        })
        .catch((err) => {
          subscriber.error(err);
        });
    });
  }

  async countRef(context: BackupClientContext): Promise<void> {
    this.logger.log('Counting reference');
    const hostDirectory = this.backupService.getHostDirectory(context.host);
    const destinationDirectory = this.backupService.getDestinationDirectory(context.host, context.currentBackupId);

    const refcnt = new ReferenceCount(hostDirectory, destinationDirectory, this.applicationConfig.poolPath);

    // Complete the refcnt with the real refcount
    const manifests = from(this.backupService.getManifests(context.host, context.currentBackupId)).pipe(
      concatMap((manifests) => from(manifests)),
      concatMap((manifest) => {
        this.logger.log(`Counting reference for ${manifest.manifestPath}`);
        return from(this.manifestService.generateRefcntFromManifest(manifest));
      }),
    );

    await this.poolChunkRefCnt.addReferenceCountToRefCnt(manifests, refcnt.backupPath);

    this.logger.log('Compact the reference to host and pool');

    // Compact the refcnt files
    this.logger.debug(`Compact ref count from ${refcnt.backupPath}`);
    try {
      await this.poolChunkRefCnt.addBackupRefcntTo(refcnt.backupPath, undefined, undefined, context.originalDate);
      await this.poolChunkRefCnt.addBackupRefcntTo(refcnt.hostPath, refcnt.backupPath, undefined, context.originalDate);
    } finally {
      this.logger.debug(`[END] Compact ref count from ${refcnt.backupPath}`);
    }
  }

  refreshCache(context: BackupClientContext, shares: string[]): Promise<RefreshCacheReply> {
    this.logger.log(`Refresh cache for all share [${shares.join(',')}]`);
    const request$ = fromIx(shares).pipe(
      mapIx((s) => Buffer.from(s)),
      mapIx((share) => {
        this.logger.log(`Refresh cache for ${share.toString()}`);
        const manifest = this.backupService.getManifest(context.host, context.currentBackupId, share);
        return concatIx(
          ofIx({ header: { sharePath: share }, fileManifest: undefined } as RefreshCacheRequest),
          this.manifestService
            .readManifestEntries(manifest)
            .pipe(mapIx((fileManifest) => ({ header: undefined, fileManifest } as RefreshCacheRequest))),
          // ofIx(() => {
          //   this.logger.log(`End of refreshing cache for ${share.toString()}`);
          //   return { header: undefined, fileManifest: undefined };
          // }),
        );
      }),
      concatAll<RefreshCacheRequest>(),
    );

    return this.#getClientInterface(context).refreshCache(context, request$);
  }

  close(context: BackupClientContext): void {
    this.logger.log(`Close connection (${context.sessionId})`);
    try {
      // Stop subscription
      context.abortable.forEach((s) => s.abort());
      context.abortable = [];
      context.logger?.close();
      context.logger = undefined;

      // Close connection
      context.sessionId = undefined;

      this.#getClientInterface(context).close(context);

      this.logger.log(`Connection closed (${context.sessionId})`);
    } catch (err) {
      this.logger.error(`Error closing connection (${context.sessionId})`, err);
    }
  }
}
