import { Injectable } from '@nestjs/common';
import { join } from 'path';

@Injectable()
export class Manifest {
  public readonly journalPath: string;
  public readonly manifestPath: string;
  public readonly newPath: string;
  public readonly lockPath: string;

  constructor(manifestName: string, private path: string) {
    this.journalPath = join(path, `${manifestName}.journal`);
    this.manifestPath = join(path, `${manifestName}.manifest`);
    this.newPath = join(path, `${manifestName}.new`);
    this.lockPath = join(path, `${manifestName}.lock`);
  }
}
