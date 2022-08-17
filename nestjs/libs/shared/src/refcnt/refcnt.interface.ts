import { Injectable } from '@nestjs/common';
import { from } from 'ix/asynciterable';
import { join } from 'path';
import { PoolRefCount, PoolUnused } from '../shared';
import { pick } from '../utils';

export enum ReferenceCountFileTypeEnum {
  HOST = 'host',
  BACKUP = 'backup',
  POOL = 'pool',
}

@Injectable()
export class ReferenceCount {
  public readonly hostPath: string;
  public readonly backupPath: string;
  public readonly poolPath: string;
  public readonly unusedPoolPath: string;

  constructor(hostPath: string, backupPath: string, poolPath: string) {
    this.backupPath = join(backupPath, `REFCNT.backup`);
    this.hostPath = join(hostPath, `REFCNT.host`);
    this.poolPath = join(poolPath, `REFCNT.pool`);
    this.unusedPoolPath = join(poolPath, `REFCNT.unused`);
  }

  getPaths() {
    return {
      [ReferenceCountFileTypeEnum.BACKUP]: this.backupPath,
      [ReferenceCountFileTypeEnum.HOST]: this.hostPath,
      [ReferenceCountFileTypeEnum.POOL]: this.poolPath,
    };
  }
}

export class SetOfPoolUnused {
  private unusedPoolMap = new Map<string, PoolUnused>();

  static async fromIterator(it: AsyncIterable<PoolUnused>) {
    const set = new SetOfPoolUnused();
    for await (const unused of it) {
      set.add(unused);
    }
    return set;
  }

  static async fromMapPoolRefCount(it: AsyncIterable<PoolRefCount>) {
    const set = new SetOfPoolUnused();
    for await (const unused of it) {
      set.add(pick(unused, 'sha256', 'compressedSize', 'size'));
    }
    return set;
  }

  toIterator() {
    return from(this.unusedPoolMap.values());
  }

  has(unused: PoolUnused | PoolRefCount): boolean {
    return this.unusedPoolMap.has(unused.sha256.toString('base64'));
  }

  delete(unused: PoolUnused | PoolRefCount): boolean {
    return this.unusedPoolMap.delete(unused.sha256.toString('base64'));
  }

  add(unused: PoolUnused | PoolRefCount) {
    this.unusedPoolMap.set(unused.sha256.toString('base64'), pick(unused, 'sha256', 'compressedSize', 'size'));
  }

  get size() {
    return this.unusedPoolMap.size;
  }
}
