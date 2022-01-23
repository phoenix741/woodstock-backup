import { Injectable } from '@nestjs/common';
import { PoolRefCount, RefCntService, ReferenceCount } from '@woodstock/shared';
import { ApplicationConfigService } from '../../config/application-config.service';

export type ChunkRefCnt = Record<string, number>;

@Injectable()
export class PoolChunkRefCnt {
  constructor(private applicationConfig: ApplicationConfigService, private refcntService: RefCntService) {}

  get poolPath(): string {
    return this.applicationConfig.poolPath;
  }

  async writeJournal(
    source: AsyncIterable<PoolRefCount>,
    backupPath: string,
    mapping?: (v: PoolRefCount) => Promise<PoolRefCount | undefined>,
  ): Promise<void>;
  async writeJournal<T = PoolRefCount>(
    source: AsyncIterable<T>,
    backupPath: string,
    mapping: (v: T) => Promise<PoolRefCount | undefined>,
  ): Promise<void>;
  async writeJournal<T = PoolRefCount>(
    source: AsyncIterable<T>,
    backupPath: string,
    mapping: (v: T) => Promise<PoolRefCount | undefined> = (v) => v as any,
  ): Promise<void> {
    const refcnt = new ReferenceCount(backupPath, backupPath, this.poolPath);
    await this.refcntService.writeRefCnt(source, refcnt.journalPath, true, mapping);
  }

  async compact(refcnt: ReferenceCount) {
    return this.refcntService.compactRefCnt(refcnt);
  }
}
