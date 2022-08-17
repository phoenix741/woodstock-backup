import { Injectable } from '@nestjs/common';
import { join } from 'path';

@Injectable()
export class Manifest {
  readonly fileListPath: string;
  readonly journalPath: string;
  readonly manifestPath: string;
  readonly newPath: string;
  readonly lockPath: string;

  constructor(manifestName: string, path: string) {
    this.fileListPath = join(path, `${manifestName}.filelist`);
    this.journalPath = join(path, `${manifestName}.journal`);
    this.manifestPath = join(path, `${manifestName}.manifest`);
    this.newPath = join(path, `${manifestName}.new`);
    this.lockPath = join(path, `${manifestName}.lock`);
  }
}
