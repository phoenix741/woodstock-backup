import { Injectable, Logger } from '@nestjs/common';
import { concat, from, reduce, toSet } from 'ix/asynciterable';
import { catchError, map } from 'ix/asynciterable/operators';
import { dirname } from 'path';
import { lock } from 'proper-lockfile';
import { PoolRefCount, PoolStatistics, PoolUnused } from '../models';
import { ProtoPoolRefCount, ProtoPoolUnused } from '../models/object-proto.model.js';
import { ProtobufService } from '../services/protobuf.service.js';
import { PoolStatisticsService } from '../statistics/pool-statistics.service.js';

@Injectable()
export class RefCntService {
  private logger = new Logger(RefCntService.name);

  constructor(
    private readonly protobufService: ProtobufService,
    private readonly statsService: PoolStatisticsService,
  ) {}

  /* ******** Write and read reference count ************ */

  async writeRefCnt(source: AsyncIterable<PoolRefCount>, filename: string): Promise<void> {
    this.logger.debug(`Write ref count into ${filename}`);
    await this.protobufService.atomicWriteFile<PoolRefCount>(filename, ProtoPoolRefCount, source);
  }

  /**
   * This method add only chunk information in the refcnt file. It is used to ensure that the refcnt
   * file contains only chunks information and not refCount number.
   * @param source The information to add to the current refcnt file
   * @param filename The path of the refcnt file
   * @returns Promise that complete when the operation is done
   */
  async addChunkInformationToRefCnt(source: AsyncIterable<PoolRefCount>, filename: string): Promise<void> {
    this.logger.debug(`Write empty ref count into ${filename}`);
    const originRefcnt = this.readRefCnt(filename);

    await this.writeRefCnt(from(concat(originRefcnt, source)).pipe(map((v) => ({ ...v, refCount: 0 }))), filename);
  }

  /**
   * This method is used to add the reference count to the refcnt file. The refcnt should already have
   * the chunk information (size, compressedsize).
   */
  async addReferenceCountToRefCnt(source: AsyncIterable<PoolRefCount>, filename: string): Promise<void> {
    this.logger.debug(`Add reference count to ${filename}`);
    const originRefcnt = this.readRefCnt(filename);

    await this.writeRefCnt(concat(originRefcnt, source), filename);
  }

  async fixRefcnt(source: AsyncIterable<PoolRefCount>, filename: string): Promise<void> {
    this.logger.debug(`Merge reference count to ${filename}`);
    const originRefcnt = this.readRefCnt(filename);

    await this.writeRefCnt(concat(from(originRefcnt).pipe(map((v) => ({ ...v, refCount: 0 }))), source), filename);

    // And compact
    await this.addBackupRefcntTo(filename);
  }

  readRefCnt(filename: string, compress = true): AsyncIterable<PoolRefCount> {
    this.logger.debug(`Read ref count from ${filename}`);
    return from(this.protobufService.loadFile<PoolRefCount>(filename, ProtoPoolRefCount, compress)).pipe(
      map((v) => v.message),
      catchError((err) => {
        this.logger.warn("Can't read the file :" + err.message);
        return from([]);
      }),
    );
  }

  async readAllRefCnt(filename: string): Promise<Map<string, PoolRefCount>> {
    const fileToChange = this.readRefCnt(filename);

    const statistics: PoolStatistics = {
      longestChain: 0,
      nbChunk: 0,
      nbRef: 0,
      size: 0n,
      compressedSize: 0n,
    };

    return await this.#calculateRefCount(fileToChange, statistics);
  }

  /** ******************* Write and read file of unused chunk **************** */

  async writeUnused(source: AsyncIterable<PoolUnused>, filename: string): Promise<void> {
    this.logger.debug(`Write unused into ${filename}`);
    await this.protobufService.atomicWriteFile<PoolUnused>(filename, ProtoPoolUnused, source);
  }

  readUnused(filename: string): AsyncIterable<PoolUnused> {
    this.logger.debug(`Read unused from ${filename}`);
    return from(this.protobufService.loadFile<PoolUnused>(filename, ProtoPoolUnused)).pipe(
      map((v) => v.message),
      catchError((err) => {
        this.logger.warn("Can't read the file :" + err.message);
        return from([]);
      }),
    );
  }

  /** ************************* Add reference counting in host and pool file ******************** */

  async #calculateRefCount(
    it: AsyncIterable<PoolRefCount>,
    statistics: PoolStatistics,
    unusedArray = new Set<string>(),
    seed = new Map<string, PoolRefCount>(),
  ) {
    const reducedRefCount = await reduce(it, {
      callback: (acc, v) => {
        const shaStr = v.sha256.toString('base64');
        const cnt = acc.get(shaStr);

        const refCount = (cnt?.refCount || 0) + v.refCount;

        if (cnt?.compressedSize !== v.compressedSize && !!cnt?.compressedSize && !!v.compressedSize) {
          this.logger.warn(`Registered compressed size is different for ${shaStr}`);
        }
        if (cnt?.compressedSize !== v.compressedSize && !!cnt?.compressedSize && !!v.compressedSize) {
          this.logger.warn(`Registered size is different for ${shaStr}`);
        }

        acc.set(shaStr, {
          sha256: v.sha256,
          refCount,
          compressedSize: cnt?.compressedSize || v.compressedSize,
          size: cnt?.size || v.size,
        });

        return acc;
      },
      seed,
    });

    for (const [key, obj] of reducedRefCount.entries()) {
      statistics.nbRef += Math.max(obj.refCount, 0);

      if (obj.refCount > statistics.longestChain) {
        statistics.longestChain = obj.refCount;
      }

      if (obj.refCount > 0) {
        if (unusedArray.has(key)) {
          unusedArray.delete(key);
        }

        statistics.size += BigInt(obj.size);
        statistics.compressedSize += BigInt(obj.compressedSize);
      } else if (obj.refCount <= 0) {
        if (obj.refCount < 0) {
          this.logger.warn(`Invalid ref count for ${obj.sha256.toString('hex')}: can't be negative`);
        }
        unusedArray.add(key);

        reducedRefCount.delete(key);
      }
    }

    statistics.nbChunk = reducedRefCount.size;

    return reducedRefCount;
  }

  async addBackupRefcntTo(fileToChangePath: string, backupRefcntPath?: string, unusedPath?: string): Promise<void> {
    this.logger.debug(`Add ${backupRefcntPath} ref count to ${fileToChangePath}`);
    const unlock = await lock(fileToChangePath, { realpath: false });
    try {
      const fileToUpdate = this.readRefCnt(fileToChangePath);
      const backupRefcnt = backupRefcntPath ? this.readRefCnt(backupRefcntPath) : from([]);
      const unused = unusedPath ? this.readUnused(unusedPath) : from([]);

      const statistics = new PoolStatistics();

      const unusedArray = await toSet(from(unused).pipe(map((v) => v.sha256.toString('base64'))));

      const rrefcnt = await this.#calculateRefCount(concat(fileToUpdate, backupRefcnt), statistics, unusedArray);

      if (unusedPath) {
        await this.writeUnused(
          from(unusedArray).pipe(map(async (v) => ({ sha256: Buffer.from(v, 'base64') }))),
          unusedPath,
        );
      }
      await this.writeRefCnt(from(rrefcnt.values()), fileToChangePath);
      await this.statsService.writeStatistics(statistics, dirname(fileToChangePath));
    } catch (err) {
      this.logger.log(`Error while compacting ref count from ${fileToChangePath} : ${err.message}`, err);
    } finally {
      await unlock();
      this.logger.debug(`[END] Compact ref count from ${fileToChangePath}`);
    }
  }

  /** ************** File removing ************************* */

  async removeBackupRefcntTo(fileToChangePath: string, backupRefcntPath: string, unusedPath?: string): Promise<void> {
    this.logger.debug(`Remove ${backupRefcntPath} ref count to ${fileToChangePath}`);
    const unlock = await lock(fileToChangePath, { realpath: false });
    try {
      const fileToChange = this.readRefCnt(fileToChangePath);
      const backupRefcnt = from(this.readRefCnt(backupRefcntPath)).pipe(map((v) => ({ ...v, refCount: -v.refCount })));
      const unused = unusedPath ? this.readUnused(unusedPath) : from([]);

      const unusedArray = await toSet(from(unused).pipe(map((v) => v.sha256.toString('base64'))));

      const originalCount = await this.#calculateRefCount(fileToChange, new PoolStatistics(), unusedArray);

      const statistics = new PoolStatistics();
      const rrefcnt = await this.#calculateRefCount(backupRefcnt, statistics, unusedArray, originalCount);

      if (unusedPath) {
        await this.writeUnused(
          from(unusedArray).pipe(map(async (v) => ({ sha256: Buffer.from(v, 'base64') }))),
          unusedPath,
        );
      }
      await this.writeRefCnt(from(rrefcnt.values()), fileToChangePath);
      await this.statsService.writeStatistics(statistics, dirname(fileToChangePath));
    } catch (err) {
      this.logger.error(`Error while cleaning up ref count : ${err.message}`, err);
    } finally {
      await unlock();
      this.logger.debug(`[END] Cleanup ref count from ${fileToChangePath} done`);
    }
  }

  async deleteRefcnt(refcnt: string): Promise<void> {
    await this.protobufService.rmFile(refcnt);
  }
}
