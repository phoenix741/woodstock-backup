import { Injectable, LoggerService } from '@nestjs/common';
import { AsyncIterableX, concat, count, from, reduce } from 'ix/asynciterable';
import { concatMap, filter, map, tap } from 'ix/asynciterable/operators';
import { from as fromRx, Observable } from 'rxjs';
import { ApplicationConfigService, BackupsService, HostsService } from '../config';
import { ManifestChunk, ManifestService } from '../manifest';
import { PoolService } from '../pool';
import { RefCntService, ReferenceCount, SetOfPoolUnused } from '../refcnt';
import { FileManifest, PoolRefCount } from '../shared';
import { QueueTaskProgression } from '../tasks';

interface ManifestChunkCount {
  count: number;
  manifests: FileManifest[];
}

export interface RefcntError {
  sha256: string;
  originalRefcnt: number;
  newRefcnt: number;
  filename: Buffer[];
}

export type LoggingFunction<T = number | bigint> = (progress: T, count: T, message: string) => void;
export type ErrorFunction = (message: string) => void;

export interface RefcntLoggger<T> {
  log: LoggingFunction<T>;
  error: ErrorFunction;
}

function isManifestChunk(chunk: ManifestChunk | PoolRefCount): chunk is ManifestChunk {
  return (chunk as ManifestChunk).manifest !== undefined;
}

@Injectable()
export class FsckService {
  constructor(
    private configService: ApplicationConfigService,
    private hostService: HostsService,
    private backupsService: BackupsService,
    private manifestService: ManifestService,
    private refCntService: RefCntService,
    private poolService: PoolService,
  ) {}

  /**
   * Read the iterator and return a map of sha256 and the number of occurence of the sha256 in the iterator
   *
   * @param chunks The iterator to read
   * @returns A map of sha256 and the number of occurence of the sha256 in the iterator
   */
  async #iterableToMap(chunks: AsyncIterableX<ManifestChunk | PoolRefCount>): Promise<Map<string, ManifestChunkCount>> {
    return await reduce(chunks, {
      callback: async (acc, chunk) => {
        const sha256Str = chunk.sha256.toString('base64');
        const ref = acc.get(sha256Str) ?? {
          count: 0,
          manifests: [],
        };

        if (isManifestChunk(chunk)) {
          ref.count += 1;
          ref.manifests.push(chunk.manifest);
        } else {
          ref.count += chunk.refCount;
        }

        acc.set(sha256Str, ref);

        return acc;
      },
      seed: new Map<string, ManifestChunkCount>(),
    });
  }

  /**
   * Read the Reference count of a backup, from the manifest (and not the reference count) and return a map of sha256 and the
   * number of occurence of the sha256 in the iterator
   *
   * @param host The host to read
   * @param backupNumber The backup number to read
   * @returns A map of sha256 and the number of occurence of the sha256 in the iterator
   */
  async #calculateRefcntFromBackup(host: string, backupNumber: number): Promise<Map<string, ManifestChunkCount>> {
    // Read backup
    const chunks = from(this.backupsService.getManifests(host, backupNumber)).pipe(
      concatMap((manifest) => from(manifest)),
      concatMap(async (manifest) => this.manifestService.listChunksFromManifest(manifest)),
    );

    return await this.#iterableToMap(chunks);
  }

  /**
   * Calculate the reference count of a host, from the reference count of each backup.
   *
   * @param host The host to read
   * @returns A map of sha256 and the number of occurence of the sha256 in the iterator
   */
  async #calculateHostRefcntFromBackupRefcnt(host: string): Promise<Map<string, ManifestChunkCount>> {
    const chunks = from(this.backupsService.getBackups(host)).pipe(
      concatMap((backup) => from(backup)),
      concatMap(async (backup) => {
        const refcnt = new ReferenceCount(
          this.backupsService.getHostDirectory(host),
          this.backupsService.getDestinationDirectory(host, backup.number),
          this.configService.poolPath,
        );

        return await this.refCntService.readRefCnt(refcnt.backupPath);
      }),
    );

    return this.#iterableToMap(chunks);
  }

  /**
   * Calculate the reference count of the pool, from the reference count of each host.
   *
   * @returns A map of sha256 and the number of occurence of the sha256 in the iterator
   */
  async #calculatePoolRefcntFromHostRefcnt(): Promise<Map<string, ManifestChunkCount>> {
    const chunks = from(this.hostService.getHosts()).pipe(
      concatMap((host) => from(host)),
      concatMap(async (host) => {
        const refcnt = new ReferenceCount(this.backupsService.getHostDirectory(host), '', this.configService.poolPath);

        return await this.refCntService.readRefCnt(refcnt.hostPath);
      }),
    );

    return this.#iterableToMap(chunks);
  }

  /**
   * Check the integrity of the reference count of from the REFCNT file compared to the map of sha256 and reference count.
   *
   * @param path The path to the REFCNT file
   * @param refcnt The map of sha256 and reference count
   * @returns An iterator of error
   */
  #checkIntegrity(path: string, refcnt: Map<string, ManifestChunkCount>): AsyncIterableX<RefcntError> {
    return from(this.refCntService.readAllRefCnt(path)).pipe(
      concatMap((originalReferenceCount) => {
        const wrongReferenceCount = from(refcnt.entries()).pipe(
          map(([sha256, ref]) => ({
            sha256,
            originalRefcnt: originalReferenceCount.get(sha256)?.refCount ?? 0,
            newRefcnt: ref.count,
            filename: ref.manifests.map((manifest) => manifest.path),
          })),
          filter((r) => r.originalRefcnt !== r.newRefcnt),
        );

        const extraReferenceCount = from(originalReferenceCount.entries()).pipe(
          filter(([key]) => !refcnt.has(key)),
          map(([sha256, ref]) => ({
            sha256,
            originalRefcnt: ref.refCount,
            newRefcnt: 0,
            filename: [],
          })),
        );

        return concat(wrongReferenceCount, extraReferenceCount);
      }),
    );
  }

  async checkBackupIntegrity(
    logger: LoggerService,
    host: string,
    number: number,
    dryRun: boolean,
  ): Promise<QueueTaskProgression> {
    const refcnt = new ReferenceCount(
      this.backupsService.getHostDirectory(host),
      this.backupsService.getDestinationDirectory(host, number),
      this.configService.poolPath,
    );

    const refcntValues = await this.#calculateRefcntFromBackup(host, number);

    const integrity = await count(
      this.#checkIntegrity(refcnt.backupPath, refcntValues).pipe(
        tap((error) => {
          logger.error(
            `${Buffer.from(error.sha256, 'base64').toString('hex')}: have ${error.originalRefcnt}, should ${
              error.newRefcnt
            }`,
          );
        }),
      ),
    );

    if (!dryRun && integrity) {
      logger.error(`Fix ${refcnt.backupPath}`);
      const source = from(refcntValues.entries()).pipe(
        map(([sha256, v]) => ({
          sha256: Buffer.from(sha256, 'base64'),
          refCount: v.count,
          size: 0,
          compressedSize: 0,
        })),
      );
      await this.refCntService.fixRefcnt(source, refcnt.backupPath);
    }

    return new QueueTaskProgression({
      fileCount: refcntValues.size,
      errorCount: integrity,
      progressCurrent: 1n,
    });
  }

  async checkHostIntegrity(logger: LoggerService, host: string, dryRun: boolean): Promise<QueueTaskProgression> {
    const refcnt = new ReferenceCount(this.backupsService.getHostDirectory(host), '', this.configService.poolPath);

    const refcntValues = await this.#calculateHostRefcntFromBackupRefcnt(host);

    const integrity = await count(
      this.#checkIntegrity(refcnt.hostPath, refcntValues).pipe(
        tap((error) => {
          logger.error(
            `${Buffer.from(error.sha256, 'base64').toString('hex')}: have ${error.originalRefcnt}, should ${
              error.newRefcnt
            }`,
          );
        }),
      ),
    );

    if (!dryRun && integrity) {
      logger.error(`Fix ${refcnt.hostPath}`);
      const source = from(refcntValues.entries()).pipe(
        map(([sha256, v]) => ({
          sha256: Buffer.from(sha256, 'base64'),
          refCount: v.count,
          size: 0,
          compressedSize: 0,
        })),
      );
      await this.refCntService.fixRefcnt(source, refcnt.hostPath);
    }

    return new QueueTaskProgression({
      fileCount: refcntValues.size,
      errorCount: integrity,
      progressCurrent: 1n,
    });
  }

  async checkPoolIntegrity(logger: LoggerService, dryRun: boolean): Promise<QueueTaskProgression> {
    const refcnt = new ReferenceCount('', '', this.configService.poolPath);

    const refcntValues = await this.#calculatePoolRefcntFromHostRefcnt();

    const integrity = await count(
      this.#checkIntegrity(refcnt.poolPath, refcntValues).pipe(
        tap((error) => {
          logger.error(
            `${Buffer.from(error.sha256, 'base64').toString('hex')}: have ${error.originalRefcnt}, should ${
              error.newRefcnt
            }`,
          );
        }),
      ),
    );

    if (!dryRun && integrity) {
      logger.error(`Fix ${refcnt.poolPath}`);
      const source = from(refcntValues.entries()).pipe(
        map(([sha256, v]) => ({
          sha256: Buffer.from(sha256, 'base64'),
          refCount: v.count,
          size: 0,
          compressedSize: 0,
        })),
      );
      await this.refCntService.fixRefcnt(source, refcnt.poolPath);
    }

    return new QueueTaskProgression({
      fileCount: refcntValues.size,
      errorCount: integrity,
      progressCurrent: 1n,
    });
  }

  async countAllChunks(): Promise<number> {
    return count(this.poolService.readAllChunks());
  }

  processUnused(logger: LoggerService, dryRun = false): Observable<QueueTaskProgression> {
    return new Observable((subscriber) => {
      (async () => {
        // Read unused file
        const refcnt = new ReferenceCount('', '', this.configService.poolPath);
        const unusedIt = this.refCntService.readUnused(refcnt.unusedPoolPath);
        const unusedPool = await SetOfPoolUnused.fromIterator(unusedIt);

        // Read Refcnt
        subscriber.next(new QueueTaskProgression({ progressMax: 1n }));

        // Read reference count file
        const refcntPool = await SetOfPoolUnused.fromMapPoolRefCount(this.refCntService.readRefCnt(refcnt.poolPath));

        const max = refcntPool.size;

        subscriber.next(new QueueTaskProgression({ progressMax: BigInt(max) }));

        // Search all file pool directory
        const files = this.poolService.readAllChunks();
        const filesSet = new SetOfPoolUnused();

        // Checking chunk in unused files
        subscriber.next(new QueueTaskProgression({ progressCurrent: 2n, progressMax: BigInt(max) }));

        // If file is not in reference count file add it to unused file (and warn if not already)
        let inUnused = 0;
        let inRefcnt = 0;
        let inNothing = 0;
        let missing = 0;
        for await (const file of files) {
          const { sha256, compressedSize } = await file.getChunkInformation(false);
          const chunk = { sha256, compressedSize: Number(compressedSize), size: 0 };
          filesSet.add(chunk);
          if (unusedPool.has(chunk)) {
            inUnused++;
            if (refcntPool.has(chunk)) {
              inRefcnt++;
              unusedPool.delete(chunk);

              logger.error(file.sha256Str + ' is in unused and in refcnt');
            }
          } else if (refcntPool.has(chunk)) {
            inRefcnt++;
          } else {
            inNothing++;
            unusedPool.add(chunk);
            logger.error(file.sha256Str + ' is not in unused nor in refcnt');
          }

          subscriber.next(new QueueTaskProgression({ progressCurrent: BigInt(inRefcnt), progressMax: BigInt(max) }));
        }

        for await (const file of refcntPool.toIterator()) {
          if (!filesSet.has(file)) {
            missing++;
          }
        }

        if (!dryRun) {
          await this.refCntService.writeUnused(unusedPool.toIterator(), refcnt.unusedPoolPath);
        }

        subscriber.next(
          new QueueTaskProgression({
            progressCurrent: BigInt(inRefcnt),
            progressMax: BigInt(max),
            newFileCount: missing + inNothing,
          }),
        );
        logger.log(
          `inUnused: ${inUnused}, inRefcnt: ${inRefcnt}, inNothing: ${inNothing}, missing: ${missing}`,
          'refcnt_unused',
        );
      })()
        .then(() => subscriber.complete())
        .catch((err) => subscriber.error(err));
    });
  }

  processVerifyChunk(logger: LoggerService): Observable<QueueTaskProgression> {
    // Read unused file
    let chunkOk = 0;
    let chunkKo = 0;

    // Search all file pool directory
    return fromRx(
      this.poolService.readAllChunks().pipe(
        map(async (chunk) => {
          const chunkInformation = await chunk.getChunkInformation();
          if (!chunk.sha256) {
            chunkKo++;
            logger.error(`${chunk.sha256Str}: chunk invalid`);
          } else {
            if (chunkInformation.sha256.equals(chunk.sha256)) {
              chunkOk++;
            } else {
              chunkKo++;
              logger.error(`${chunk.sha256Str}: chunk is corrupted`);
            }
          }

          return new QueueTaskProgression({
            errorCount: chunkKo,
            fileCount: chunkOk + chunkKo,
            progressCurrent: BigInt(chunkOk + chunkKo),
          });
        }),
      ),
    );
  }

  async checkCompression(logger: RefcntLoggger<bigint>, all?: boolean) {
    const { fileTypeFromStream } = await import('file-type');
    // Read unused file
    let compressedSize = 0n;
    let uncompressedSize = 0n;

    const refcnt = new ReferenceCount('', '', this.configService.poolPath);
    const poolRefcnt = !all
      ? this.refCntService.readRefCnt(refcnt.poolPath)
      : this.poolService.readAllChunks().pipe(
          map(async (chunk) => await chunk.getChunkInformation()),
          map(async (chunkInformation) => ({
            sha256: chunkInformation.sha256,
            refCount: 0,
            size: Number(chunkInformation.size),
            compressedSize: Number(chunkInformation.compressedSize),
          })),
        );

    for await (const r of poolRefcnt) {
      compressedSize += BigInt(r.compressedSize);
      uncompressedSize += BigInt(r.size);

      if (compressedSize > uncompressedSize) {
        const fileType = await fileTypeFromStream(this.poolService.getChunk(r.sha256).read());
        logger.error(
          `${r.sha256.toString('hex')}: compressed size is greater than uncompressed size: ${fileType?.mime}`,
        );
      }

      logger.log(
        compressedSize,
        uncompressedSize,
        `${compressedSize.toLocaleString()} compressed, ${uncompressedSize.toLocaleString()} uncompressed`,
      );
    }

    return { compressedSize, uncompressedSize };
  }
}
