import { Injectable } from '@nestjs/common';
import { join } from 'path';

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
