import { Injectable } from '@nestjs/common';
import { AsyncIterableX, concat, count, from, reduce, toSet } from 'ix/asynciterable';
import { concatMap, filter, map, tap } from 'ix/asynciterable/operators';
import { basename } from 'path';
import { ApplicationConfigService } from '../config';
import { FileBrowserService } from '../file';
import { ManifestService } from '../manifest';
import { FileManifest, PoolRefCount } from '../models';
import { ManifestChunk } from '../models/manifest.dto';
import { ReferenceCount } from '../refcnt/refcnt.model';
import { RefCntService } from '../refcnt/refcnt.service';
import { BackupsService } from './backups.service';
import { HostsService } from './hosts.service';

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

export type LoggingFunction = (progress: number, count: number, message: string) => void;
export type ErrorFunction = (message: string) => void;

export interface RefcntLoggger {
  log: LoggingFunction;
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
    private fileBrowserService: FileBrowserService,
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

  async processRefcnt(logger: RefcntLoggger, dryRun = false) {
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

  async processUnused(logger: RefcntLoggger, dryRun = false) {
    // Read unused file
    const refcnt = new ReferenceCount('', '', this.configService.poolPath);
    const unusedPool = await toSet(
      from(this.refCntService.readUnused(refcnt.unusedPoolPath)).pipe(map((chunk) => chunk.sha256.toString('base64'))),
    );

    logger.log(0, 1, `Read Refcnt`);

    // Read reference count file
    const refcntPool = await this.refCntService.readAllRefCnt(refcnt.poolPath);

    const max = refcntPool.size;

    logger.log(0, max, `Read all chunks`);
    // Search all file pool directory
    const files = await this.fileBrowserService
      .getFilesRecursive(Buffer.from(this.configService.poolPath))(Buffer.from(''))
      .pipe(
        map((file) => basename(file.toString())),
        filter((file) => file.endsWith('-sha256.zz')),
        map((file) => Buffer.from(file.substring(0, file.length - 10), 'hex').toString('base64')),
      );
    const filesSet = new Set<string>();

    logger.log(2, max, `Checking chunk in unused files`);
    //     If file is not in reference count file add it to unused file (and warn if not already)
    let inUnused = 0;
    let inRefcnt = 0;
    let inNothing = 0;
    let missing = 0;
    for await (const file of files) {
      filesSet.add(file);
      if (unusedPool.has(file)) {
        inUnused++;
        if (refcntPool.has(file)) {
          inRefcnt++;
          unusedPool.delete(file);
          logger.error(Buffer.from(file, 'base64').toString('hex') + ' is in unused and in refcnt');
        }
      } else if (refcntPool.has(file)) {
        inRefcnt++;
      } else {
        inNothing++;
        unusedPool.add(file);
        logger.error(Buffer.from(file, 'base64').toString('hex') + ' is not in unused nor in refcnt');
      }
      logger.log(inRefcnt, max, `Search for unused chunks`);
    }

    for (const file of refcntPool.keys()) {
      if (!filesSet.has(file)) {
        missing++;
      }
    }

    if (!dryRun) {
      const unusedStream = from(unusedPool.values()).pipe(map((file) => ({ sha256: Buffer.from(file, 'base64') })));
      await this.refCntService.writeUnused(unusedStream, refcnt.unusedPoolPath);
    }

    return { inUnused, inRefcnt, inNothing, missing };
  }
}
