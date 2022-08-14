import { Injectable } from '@nestjs/common';
import { AsyncIterableX, concat, count, from, reduce, toSet } from 'ix/asynciterable';
import { concatMap, filter, map, tap } from 'ix/asynciterable/operators';
import { basename } from 'path';
import { ApplicationConfigService } from '../config';
import { FileBrowserService } from '../file';
import { ManifestService } from '../manifest';
import { FileManifest, PoolRefCount } from '../models';
import { ManifestChunk } from '../models/manifest.dto.js';
import { ReferenceCount, SetOfPoolUnused } from '../refcnt/refcnt.model.js';
import { RefCntService } from '../refcnt/refcnt.service.js';
import { BackupsService } from './backups.service.js';
import { HostsService } from './hosts.service.js';
import { PoolService } from './pool/pool.service.js';

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

  async #reduceChunk(chunks: AsyncIterableX<ManifestChunk | PoolRefCount>) {
    return await reduce(chunks, {
      callback: async (acc, chunk) => {
        const ref = acc.get(chunk.sha256.toString('base64')) || {
          count: 0,
          manifests: [],
        };

        if (isManifestChunk(chunk)) {
          ref.count += 1;
          ref.manifests.push(chunk.manifest);
        } else {
          ref.count += chunk.refCount;
        }

        acc.set(chunk.sha256.toString('base64'), ref);

        return acc;
      },
      seed: new Map<string, ManifestChunkCount>(),
    });
  }

  async #refcntFromBackup(host: string, backupNumber: number): Promise<Map<string, ManifestChunkCount>> {
    // Read backup
    const chunks = from(this.backupsService.getManifests(host, backupNumber)).pipe(
      concatMap((manifest) => from(manifest)),
      concatMap(async (manifest) => this.manifestService.listChunksFromManifest(manifest)),
    );

    return await this.#reduceChunk(chunks);
  }

  async #hostRefcntFromBackup(host: string): Promise<Map<string, ManifestChunkCount>> {
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

    return this.#reduceChunk(chunks);
  }

  async #poolRefcntFromHost(): Promise<Map<string, ManifestChunkCount>> {
    const chunks = from(this.hostService.getHosts()).pipe(
      concatMap((host) => from(host)),
      concatMap(async (host) => {
        const refcnt = new ReferenceCount(this.backupsService.getHostDirectory(host), '', this.configService.poolPath);

        return await this.refCntService.readRefCnt(refcnt.hostPath);
      }),
    );

    return this.#reduceChunk(chunks);
  }

  #checkIntegrity(path: string, refcnt: Map<string, ManifestChunkCount>): AsyncIterableX<RefcntError> {
    return from(this.refCntService.readAllRefCnt(path)).pipe(
      concatMap((originalReferenceCount) => {
        const wrongReferenceCount = from(refcnt.entries()).pipe(
          map(([sha256, ref]) => ({
            sha256,
            originalRefcnt: originalReferenceCount.get(sha256)?.refCount || 0,
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

  async processRefcnt(logger: RefcntLoggger<number>, dryRun = false) {
    const hosts = await this.hostService.getHosts();

    const backups = (
      await Promise.all(
        hosts.map(async (host) =>
          (await this.backupsService.getBackups(host)).map((backup) => ({ host, number: backup.number })),
        ),
      )
    ).flat();

    const progressMax = hosts.length + backups.length + 1;
    let progress = 0;
    let errorCount = 0;

    // Start by regenerate backup refcnt
    for (let i = 0; i < backups.length; i++) {
      const backup = backups[i];
      logger.log(progress, progressMax, `Regenerating backup refcnt for ${backup.host}/${backup.number}`);

      const refcnt = new ReferenceCount(
        this.backupsService.getHostDirectory(backup.host),
        this.backupsService.getDestinationDirectory(backup.host, backup.number),
        this.configService.poolPath,
      );

      const refcntValues = await this.#refcntFromBackup(backup.host, backup.number);

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

      errorCount += integrity;
      progress += 1;
    }

    // Regenerate host refcnt
    for (let i = 0; i < hosts.length; i++) {
      const host = hosts[i];
      logger.log(progress, progressMax, `host backup refcnt for ${host}`);

      const refcnt = new ReferenceCount(this.backupsService.getHostDirectory(host), '', this.configService.poolPath);

      const refcntValues = await this.#hostRefcntFromBackup(host);

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

      errorCount += integrity;
      progress += 1;
    }

    // Regenerate pool refcnt
    logger.log(progress, progressMax, `pool refcnt`);

    const refcnt = new ReferenceCount('', '', this.configService.poolPath);

    const refcntValues = await this.#poolRefcntFromHost();

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

    errorCount += integrity;

    logger.log(progressMax, progressMax, `Refcnt checked, ${errorCount} errors`);
    return errorCount;
  }

  async processUnused(logger: RefcntLoggger<number>, dryRun = false) {
    // Read unused file
    const refcnt = new ReferenceCount('', '', this.configService.poolPath);
    const unusedIt = this.refCntService.readUnused(refcnt.unusedPoolPath);
    const unusedPool = await SetOfPoolUnused.fromIterator(unusedIt);

    logger.log(0, 1, `Read Refcnt`);

    // Read reference count file
    const refcntPool = await SetOfPoolUnused.fromMapPoolRefCount(this.refCntService.readRefCnt(refcnt.poolPath));

    const max = refcntPool.size;

    logger.log(0, max, `Read all chunks`);
    // Search all file pool directory
    const files = this.poolService.readAllChunks();
    const filesSet = new SetOfPoolUnused();

    logger.log(2, max, `Checking chunk in unused files`);
    //     If file is not in reference count file add it to unused file (and warn if not already)
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
      logger.log(inRefcnt, max, `Search for unused chunks`);
    }

    for await (const file of refcntPool.toIterator()) {
      if (!filesSet.has(file)) {
        missing++;
      }
    }

    if (!dryRun) {
      await this.refCntService.writeUnused(unusedPool.toIterator(), refcnt.unusedPoolPath);
    }

    return { inUnused, inRefcnt, inNothing, missing };
  }

  async processVerifyChunk(logger: RefcntLoggger<number>) {
    // Read unused file
    let chunkOk = 0;
    let chunkKo = 0;

    // Search all file pool directory
    const files = this.poolService.readAllChunks();

    for await (const chunk of files) {
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

      logger.log(chunkOk + chunkKo, 100, `${chunkOk} chunks ok, ${chunkKo} chunks ko`);
    }

    return { chunkOk, chunkKo };
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
