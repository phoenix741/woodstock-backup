import { Injectable, Logger } from '@nestjs/common';
import { unlink } from 'fs/promises';
import { concat, from, pipe, reduce } from 'ix/asynciterable';
import { catchError, map } from 'ix/asynciterable/operators';
import { dirname } from 'path';
import { lock } from 'proper-lockfile';
import { PoolRefCount, PoolStatistics } from '../models';
import { ProtoPoolRefCount } from '../models/object-proto.model';
import { ProtobufService } from '../services/protobuf.service';
import { PoolStatisticsService } from '../statistics/pool-statistics.service';
import { notUndefined } from '../utils/iterator.utils';
import { ReferenceCount, ReferenceCountFileTypeEnum } from './refcnt.model';

@Injectable()
export class RefCntService {
  private logger = new Logger(RefCntService.name);

  constructor(
    private readonly protobufService: ProtobufService,
    private readonly statsService: PoolStatisticsService,
  ) {}

  async writeRefCnt(
    source: AsyncIterable<PoolRefCount>,
    filename: string,
    append?: boolean,
    mapping?: (v: PoolRefCount) => Promise<PoolRefCount | undefined>,
  ): Promise<void>;
  async writeRefCnt<T = PoolRefCount>(
    source: AsyncIterable<T>,
    filename: string,
    append: boolean,
    mapping: (v: T) => Promise<PoolRefCount | undefined>,
  ): Promise<void>;
  async writeRefCnt<T = PoolRefCount>(
    source: AsyncIterable<T>,
    filename: string,
    append = false,
    mapping: (v: T) => Promise<PoolRefCount | undefined> = (v) => v as any,
  ): Promise<void> {
    this.logger.debug(`Write ref count into ${filename}`);
    const mappedSource = pipe(source, map(mapping), notUndefined());
    if (!append) {
      await this.protobufService.atomicWriteFile<PoolRefCount>(filename, ProtoPoolRefCount, mappedSource);
    } else {
      await this.protobufService.writeFile<PoolRefCount>(filename, ProtoPoolRefCount, mappedSource);
    }
  }

  readRefCnt(filename: string): AsyncIterable<PoolRefCount> {
    this.logger.debug(`Read ref count from ${filename}`);
    return pipe(
      this.protobufService.loadFile<PoolRefCount>(filename, ProtoPoolRefCount),
      map((v) => v.message),
      catchError((err) => {
        this.logger.warn("Can't read the file :" + err.message);
        return from([]);
      }),
    );
  }

  async compactRefCnt(refcnt: ReferenceCount): Promise<void> {
    this.logger.debug(`Compact ref count from ${refcnt.journalPath}`);

    try {
      for (const type in refcnt.getPaths()) {
        if (type === ReferenceCountFileTypeEnum.JOURNAL) {
          continue;
        }
        const filepath = refcnt.getPaths()[type as ReferenceCountFileTypeEnum];

        const unlock = await lock(filepath, { realpath: false });
        try {
          const fileToUpdate = this.readRefCnt(filepath);
          const journalFile = this.readRefCnt(refcnt.journalPath);

          const statistics: PoolStatistics = {
            longestChain: 0,
            nbChunk: 0,
            nbRef: 0,
            size: 0n,
            compressedSize: 0n,
          };

          const rrefcnt = await reduce(concat(fileToUpdate, journalFile), {
            callback: (acc, v) => {
              const cnt = acc.get(v.sha256.toString());
              statistics.nbRef += v.refCount;
              if (cnt) {
                const refCount = cnt.refCount + v.refCount;
                if (refCount > statistics.longestChain) {
                  statistics.longestChain = refCount;
                }
                acc.set(v.sha256.toString(), Object.assign(cnt, { refCount }));
              } else {
                if (v.refCount > statistics.longestChain) {
                  statistics.longestChain = v.refCount;
                }
                statistics.nbChunk++;
                statistics.size += BigInt(v.size);
                statistics.compressedSize += BigInt(v.compressedSize);
                acc.set(v.sha256.toString(), v);
              }
              return acc;
            },
            seed: new Map<string, PoolRefCount>(),
          });
          await this.writeRefCnt(from(rrefcnt.values()), filepath);
          if (type !== ReferenceCountFileTypeEnum.BACKUP) {
            await this.statsService.writeStatistics(statistics, dirname(filepath));
          }
        } finally {
          await unlock();
        }
      }
    } finally {
      this.logger.debug(`[END] Compact ref count from ${refcnt.journalPath}`);
      await unlink(refcnt.journalPath).catch((err) =>
        this.logger.warn("Can't delete the journal file :" + err.message),
      );
    }
  }
}
