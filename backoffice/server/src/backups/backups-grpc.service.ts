import { credentials, Metadata } from '@grpc/grpc-js';
import { Injectable, Logger, LoggerService } from '@nestjs/common';
import { ClientProxyFactory, Transport } from '@nestjs/microservices';
import {
  AuthenticateReply,
  bigIntToLong,
  ChunkInformation,
  ChunkStatus,
  EntryType,
  ExecuteCommandReply,
  FileManifest,
  FileManifestJournalEntry,
  GetChunkReply,
  joinBuffer,
  LaunchBackupReply,
  LogEntry,
  LogLevel,
  longToBigInt,
  LONG_CHUNK_SIZE,
  mangle,
  Manifest,
  ManifestService,
  RefreshCacheRequest,
  Share,
  StatusCode,
} from '@woodstock/shared';
import { readFile } from 'fs/promises';
import { asAsyncIterable } from 'ix';
import {
  concat as concatIx,
  from as fromIx,
  of as ofIx,
  pipe,
  range as rangeIx,
  reduce as reduceIx,
} from 'ix/asynciterable';
import { concatAll, filter, map as mapIx } from 'ix/asynciterable/operators';
import * as Long from 'long';
import { join } from 'path';
import { concat, defer, endWith, from, map, mapTo, Observable, reduce, scan, startWith, Subscriber, tap } from 'rxjs';
import { concurrentMap } from 'src/utils/p-promise.util';
import { Readable, Writable } from 'stream';
import { pipeline } from 'stream/promises';
import { BackupsService } from '../backups/backups.service';
import { TaskProgression } from '../tasks/tasks.dto';
import { BackupsGrpcContext } from './backups-grpc.dto';
import { PoolChunkRefCnt } from './pool/pool-chunk-refcnt';
import { isPoolChunkInformation, PoolChunkInformation } from './pool/pool-chunk.dto';
import { PoolService } from './pool/pool.service';

export const SHA256_EMPTYSTRING = Buffer.from(
  'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
  'hex',
);

export class LaunchBackupError extends Error {}

@Injectable()
export class BackupsGrpc {
  private logger = new Logger(BackupsGrpc.name);

  constructor(
    private backupService: BackupsService,
    private manifestService: ManifestService,
    private poolService: PoolService,
    private poolChunkRefCnt: PoolChunkRefCnt,
  ) {}

  private getMetadata(context: BackupsGrpcContext) {
    if (!context.sessionId) {
      throw new LaunchBackupError("Can't find the sessionId");
    }

    const metadata = new Metadata();
    metadata.add('X-Session-Id', context.sessionId);
    return metadata;
  }

  createConnection(ip: string, hostname: string, currentBackupId: number): Observable<BackupsGrpcContext> {
    return defer(async () => {
      this.logger.log(`Create connection to ${hostname} (${ip})`);
      const channel_creds = credentials.createSsl(
        await readFile('../../client/client-sync/certs/rootCA.pem'),
        await readFile('../../client/client-sync/certs/server.key'),
        await readFile('../../client/client-sync/certs/server.crt'),
      );

      const client = ClientProxyFactory.create({
        transport: Transport.GRPC,
        options: {
          package: 'woodstock',
          protoPath: join(__dirname, '..', '..', '..', '..', 'packages', 'shared', 'woodstock.proto'),
          url: ip + ':3657',
          credentials: channel_creds,
          channelOptions: {
            'grpc.ssl_target_name_override': 'pc-ulrich.eden.lan',
            'grpc.enable_channelz': 0,
            'grpc.default_compression_algorithm': 3,
            'grpc.default_compression_level': 2,
          },
        },
      });

      return new BackupsGrpcContext(hostname, ip, currentBackupId, client);
    });
  }

  authenticate(context: BackupsGrpcContext, logger: LoggerService): Observable<TaskProgression> {
    this.logger.log(`Authenticate to ${context.host} (${context.ip})`);

    const authenticate$ = new Observable<AuthenticateReply>((observer) => {
      context.service.authenticate({ version: 0 }, (err, reply) => {
        if (err) {
          observer.error(err);
          return;
        }
        observer.next(reply);
        observer.complete();
      });
    });

    return authenticate$.pipe(
      map((reply) => {
        if (!reply || reply.code === StatusCode.Failed || !reply.sessionId) {
          throw new LaunchBackupError('Authentication failed');
        }
        context.sessionId = reply.sessionId;
        context.logger = logger;
        const streamLog = from(
          context.service
            .streamLog({}, this.getMetadata(context))
            .pipe(asAsyncIterable<LogEntry>({ objectMode: true })),
        );

        context.subscriptions.push(
          streamLog.subscribe({
            next: (log) => {
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
            complete: () => {
              logger.log('The backup process has been completed', 'logger');
            },
            error: (err) => {
              logger.error(`The backup process has been failed: ${err.message}`, 'logger');
            },
          }),
        );

        this.logger.log(`Authentication has been completed: ${context.sessionId}`);
        return new TaskProgression({ percent: 100 });
      }),
      startWith(new TaskProgression({ percent: 0 })),
    );
  }

  executeCommand(context: BackupsGrpcContext, command: string): Observable<TaskProgression> {
    this.logger.log(`Execute command (${context.sessionId}): ${command}`);

    context.logger?.log(`Execute command: ${command}`, 'executeCommand');
    const executeCommand$ = new Observable<ExecuteCommandReply>((observer) => {
      context.service.executeCommand({ command }, this.getMetadata(context), (err, reply) => {
        if (err) {
          observer.error(err);
          return;
        }
        observer.next(reply);
        observer.complete();
      });
    });
    return executeCommand$.pipe(
      map((reply) => {
        reply?.stderr && context.logger?.error(reply.stderr);
        reply?.stdout && context.logger?.error(reply.stdout);

        if (!reply || reply.code) {
          context.logger?.log(
            `The command "${command}" has been executed with error: ${reply?.code}`,
            'executeCommand',
          );
          throw new LaunchBackupError(reply.stderr || `Can\' execute the command ${command}`);
        } else {
          context.logger?.log(`The command "${command}" has been executed successfully.`, 'executeCommand');
        }
        return new TaskProgression({ percent: 100 });
      }),
      startWith(new TaskProgression({ percent: 0 })),
    );
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

          if (!oldChunk.sha256.equals(sha256)) {
            this.logger.error(
              `${fileManifest.path.toString()}:${chunkNumber}: Chunk ${sha256} is not the same that ${
                oldChunk.sha256
              }.`,
            );
          }

          return oldChunk;
        } else {
          // Create the chunk
          const chunkResult = context.service.getChunk({ chunk }, this.getMetadata(context));
          const newChunk = await wrapper.write(
            Readable.from(
              pipe(
                chunkResult,
                mapIx<GetChunkReply, GetChunkReply>((pieceOfChunk) => {
                  if (pieceOfChunk.status === ChunkStatus.ERROR) {
                    throw new Error(`Can't read the chunk ${chunk.sha256}`);
                  }
                  return pieceOfChunk;
                }),
                filter<GetChunkReply>((pieceOfChunk) => pieceOfChunk.status === ChunkStatus.DATA),
                mapIx<GetChunkReply, Buffer>((pieceOfChunk) => pieceOfChunk.data.data),
              ),
            ),
          );

          return newChunk;
        }
      }

      return {
        size: 0n,
        compressedSize: 0n,
        sha256: SHA256_EMPTYSTRING,
      };
    } catch (err) {
      this.logger.error(`${fileManifest.path.toString()}:${chunkNumber}: ${(err as Error).message}`, err);
    }
  }

  private async downloadManifestFile(
    context: BackupsGrpcContext,
    sharePath: Buffer,
    fileManifest: FileManifest,
    subscriber: Subscriber<FileManifestJournalEntry | PoolChunkInformation>,
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
        subscriber.next(chunk);
        return acc;
      },
    });

    fileManifest.stats.compressedSize = bigIntToLong(manifestSize.compressedSize);
    if (!fileManifest.stats.size.equals(bigIntToLong(manifestSize.size))) {
      this.logger.error(
        `The manifest of file ${fileManifest.path.toString()} size (${fileManifest.stats.size.toString()}) is not equal to the sum of chunk size ${manifestSize.size.toString()}`,
      );
      fileManifest.stats.size = bigIntToLong(manifestSize.size);
    }
    fileManifest.chunks = manifestSize.chunks;

    return fileManifest;
  }

  private downloadChunksFromJournal(
    context: BackupsGrpcContext,
    backupShare: Share,
    manifest: Manifest,
  ): Observable<FileManifestJournalEntry | PoolChunkInformation> {
    return new Observable<FileManifestJournalEntry | PoolChunkInformation>((subscriber) => {
      const entries = this.manifestService.readFilelistEntries(manifest).pipe(
        concurrentMap(20, async (entry) => {
          try {
            if (entry?.type !== EntryType.REMOVE && entry?.manifest) {
              const manifest = await this.downloadManifestFile(
                context,
                backupShare.sharePath,
                entry.manifest,
                subscriber,
              );
              return { type: entry.type, manifest };
            } else {
              return entry;
            }
          } catch (err) {
            // FIXME: Gérer l'erreur
            this.logger.error(`${entry.manifest.path.toString()}: ${(err as Error).message}`, err);
            return undefined;
          }
        }),
      );
      this.manifestService
        .writeJournalEntry(entries, manifest, async (entry) => {
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
    });
  }

  private downloadFileList(context: BackupsGrpcContext, backupShare: Share, manifest: Manifest) {
    return new Observable<FileManifestJournalEntry>((subscriber) => {
      const grpclaunchBackup$ = context.service.launchBackup({ share: backupShare }, this.getMetadata(context));
      const launchBackup$ = fromIx<LaunchBackupReply>(grpclaunchBackup$).pipe(mapIx(({ entry }) => entry));
      this.manifestService
        .writeFileListEntry(launchBackup$, manifest, async (entry) => {
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
    });
  }

  createBackup(context: BackupsGrpcContext, backupShare: Share): Observable<TaskProgression> {
    this.logger.log(`Create backup (${context.sessionId}): ${backupShare.sharePath.toString()}`);
    const manifest = new Manifest(
      `backups-${mangle(backupShare.sharePath)}`,
      this.backupService.getDestinationDirectory(context.host, context.currentBackupId),
    );

    let count = 0n;
    const fileList$ = this.downloadFileList(context, backupShare, manifest).pipe(
      tap((v) => (count += longToBigInt(v.manifest.stats.size || Long.ZERO))),
      mapTo(new TaskProgression({ percent: 0 })),
    );

    const readChunk$ = this.downloadChunksFromJournal(context, backupShare, manifest).pipe(
      scan(
        (current, value) => {
          if (isPoolChunkInformation(value)) {
            return {
              ...current,
              progress: current.progress + value.size,
            };
          } else {
            switch (value.type) {
              case EntryType.ADD:
                return {
                  ...current,
                  count: current.count + 1,
                  size: current.size + longToBigInt(value.manifest.stats.size),
                  compressedFileSize:
                    current.compressedFileSize + longToBigInt(value.manifest.stats.compressedSize || Long.ZERO),
                };
              case EntryType.MODIFY:
                return {
                  ...current,
                  size: current.size + longToBigInt(value.manifest.stats.size),
                  compressedFileSize:
                    current.compressedFileSize + longToBigInt(value.manifest.stats.compressedSize || Long.ZERO),
                };
              case EntryType.REMOVE:
                return current;
            }
          }
        },
        {
          progress: 0n,
          count: 0,
          size: 0n,
          compressedFileSize: 0n,
          date: new Date(),
        },
      ),
      map(
        (fileCount) =>
          new TaskProgression({
            newCompressedFileSize: fileCount.compressedFileSize,
            newFileCount: fileCount.count,
            newFileSize: fileCount.size,
            percent: Number((fileCount.progress * 100n) / count),
            speed: Number((fileCount.progress * 1000n) / BigInt(Date.now() - fileCount.date.getTime())),
          }),
      ),
    );

    return concat(fileList$, readChunk$).pipe(startWith(new TaskProgression({ percent: 0 })));
  }

  countRef(context: BackupsGrpcContext, sharePath: Buffer): Observable<TaskProgression> {
    this.logger.log('Counting reference for the share: ' + sharePath.toString());
    const manifest = new Manifest(
      `backups-${mangle(sharePath)}`,
      this.backupService.getDestinationDirectory(context.host, context.currentBackupId),
    );

    const compactManifest$ = new Observable<FileManifest>((subscriber) => {
      this.manifestService
        .compact(manifest, async (v) => {
          if (v.chunks) {
            await this.poolChunkRefCnt.incrBatch(v.chunks);
          }
          if (v) {
            subscriber.next(v);
            return v;
          }
          return undefined;
        })
        .then(() => {
          subscriber.complete();
        })
        .catch((err) => {
          subscriber.error(err);
        });
    });

    const fileCount$ = compactManifest$.pipe(
      scan(
        (current, value) => {
          return {
            ...current,
            progress: current.progress + 1,
            count: current.count + 1,
            size: current.size + longToBigInt(value.stats.size),
            compressedFileSize: current.compressedFileSize + longToBigInt(value.stats.compressedSize || Long.ZERO),
          };
        },
        {
          progress: 0,
          count: 0,
          size: 0n,
          compressedFileSize: 0n,
        },
      ),
    );

    return fileCount$.pipe(
      map(
        (fileCount) =>
          new TaskProgression({
            compressedFileSize: fileCount.compressedFileSize,
            fileCount: fileCount.count,
            fileSize: fileCount.size,
            percent: 0,
          }),
      ),
      startWith(new TaskProgression({ percent: 0 })),
    );
  }

  refreshCache(context: BackupsGrpcContext, shares: string[]): Observable<TaskProgression> {
    const shares$ = fromIx(shares).pipe(mapIx((s) => Buffer.from(s)));
    const request$ = shares$.pipe(
      mapIx((share) => {
        const manifest = new Manifest(
          `backups-${mangle(share)}`,
          this.backupService.getDestinationDirectory(context.host, context.currentBackupId),
        );
        return concatIx(
          ofIx({ header: { sharePath: share }, fileManifest: undefined } as RefreshCacheRequest),
          this.manifestService
            .readManifestEntries(manifest)
            .pipe(mapIx((fileManifest) => ({ header: undefined, fileManifest } as RefreshCacheRequest))),
        );
      }),
      concatAll<RefreshCacheRequest>(),
    );

    const refreshCache$ = new Observable((subscriber) => {
      const writer = context.service.refreshCache(this.getMetadata(context), (err, response) => {
        if (err) {
          subscriber.error(err);
        } else {
          subscriber.next(response);
          subscriber.complete();
        }
      });
      pipeline(Readable.from(request$), writer as unknown as NodeJS.WritableStream).catch((err) => {
        subscriber.error(err);
      }); // FIXME: abort
    });

    return refreshCache$.pipe(
      reduce((progression) => progression, new TaskProgression()),
      startWith(new TaskProgression({ percent: 0 })),
      endWith(new TaskProgression({ percent: 100 })),
    );
  }

  close(context: BackupsGrpcContext) {
    this.logger.log(`Close connection (${context.sessionId})`);
    try {
      // Stop subscription
      context.subscriptions.forEach((s) => s.unsubscribe());
      context.subscriptions = [];
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
