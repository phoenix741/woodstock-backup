import { Injectable, Logger } from '@nestjs/common';
import { constants as constantsFs } from 'fs';
import { lstat, readdir } from 'fs/promises';
import * as Long from 'long';
import { EMPTY, from, merge, Observable } from 'rxjs';
import { catchError, mergeMap, switchMap, tap, filter } from 'rxjs/operators';

import { FileManifest } from '../models/manifest.model';
import { bigIntToLong } from '../utils/number.utils';
import { joinBuffer } from '../utils/path.utils';

@Injectable()
export class FileBrowserService {
  private logger = new Logger(FileBrowserService.name);

  public getFiles(backupPath: Buffer, includes: RegExp[], excludes: RegExp[]): Observable<FileManifest> {
    const files$ = from(readdir(backupPath, { encoding: 'buffer' })).pipe(
      catchError(() => {
        this.logger.warn(`Can't read the directory ${backupPath.toString()}`);
        return EMPTY;
      }),
      mergeMap((files) => from(files)),
      filter((file) => FileBrowserService.isFileAuthorized(file, includes, excludes)),
      mergeMap((file) =>
        from(this.createManifestFromLocalFile(joinBuffer(backupPath, file))).pipe(
          catchError((err) => {
            this.logger.warn(`Can't read the file ${backupPath.toString()}`);
            return EMPTY;
          }),
        ),
      ),
    );
    const folders$ = files$.pipe(
      filter((folder) => FileBrowserService.isDirectory(folder.stats.mode)),
      mergeMap((folder) => this.getFiles(folder.path, includes, excludes)),
    );
    return merge(files$, folders$);
  }

  private async createManifestFromLocalFile(path: Buffer): Promise<FileManifest> {
    const fileStat = await lstat(path, { bigint: true });
    return {
      path,
      stats: {
        ownerId: bigIntToLong(fileStat.uid),
        groupId: bigIntToLong(fileStat.gid),
        size: bigIntToLong(fileStat.size),
        mode: bigIntToLong(fileStat.mode),
        lastModified: bigIntToLong(fileStat.mtimeMs),
        lastRead: bigIntToLong(fileStat.atimeMs),
        created: bigIntToLong(fileStat.birthtimeMs),
      },
    };
  }

  private static isFileAuthorized(file: Buffer, includes: RegExp[], excludes: RegExp[]): boolean {
    const latin1File = file.toString('latin1');
    if (includes.length) {
      if (!includes.some((include) => include.test(latin1File))) {
        return false;
      }
    }

    if (excludes.length) {
      if (excludes.some((exclude) => exclude.test(latin1File))) {
        return false;
      }
    }

    return true;
  }

  private static isDirectory(mode: Long = Long.ZERO): boolean {
    return (mode.toNumber() & constantsFs.S_IFMT) === constantsFs.S_IFDIR;
  }
}
