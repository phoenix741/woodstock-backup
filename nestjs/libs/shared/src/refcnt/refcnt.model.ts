import { Injectable } from '@nestjs/common';
import { join } from 'path';

export enum ReferenceCountFileTypeEnum {
  JOURNAL = 'journal',
  HOST = 'host',
  BACKUP = 'backup',
  POOL = 'pool',
}

@Injectable()
export class ReferenceCount {
  public readonly journalPath: string;
  public readonly hostPath: string;
  public readonly backupPath: string;
  public readonly poolPath: string;

  constructor(hostPath: string, backupPath: string, poolPath: string) {
    this.journalPath = join(backupPath, `REFCNT.journal`);
    this.backupPath = join(backupPath, `REFCNT.backup`);
    this.hostPath = join(hostPath, `REFCNT.host`);
    this.poolPath = join(poolPath, `REFCNT.pool`);
  }

  getPaths() {
    return {
      [ReferenceCountFileTypeEnum.JOURNAL]: this.journalPath,
      [ReferenceCountFileTypeEnum.BACKUP]: this.backupPath,
      [ReferenceCountFileTypeEnum.HOST]: this.hostPath,
      [ReferenceCountFileTypeEnum.POOL]: this.poolPath,
    };
  }
}
