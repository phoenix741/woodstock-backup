import { Injectable, Logger } from '@nestjs/common';
import { constants as constantsFs } from 'fs';
import { lstat, readdir } from 'fs/promises';
import * as Long from 'long';
import { EMPTY, from, merge, Observable } from 'rxjs';
import { catchError, mergeMap, switchMap, tap, filter, concatMap } from 'rxjs/operators';

import { FileManifest } from '../models/manifest.model';
import { bigIntToLong } from '../utils/number.utils';
import { joinBuffer } from '../utils/path.utils';

@Injectable()
export class FileBrowserService {
  private logger = new Logger(FileBrowserService.name);

  public getFiles(
    sharePath: Buffer,
  ): (backupPath: Buffer, includes: RegExp[], excludes: RegExp[]) => Observable<FileManifest> {
    const forShare = (backupPath: Buffer, includes: RegExp[], excludes: RegExp[]): Observable<FileManifest> => {
      const files$ = from(readdir(joinBuffer(sharePath, backupPath), { encoding: 'buffer' })).pipe(
        catchError(() => {
          this.logger.warn(`Can't read the directory ${sharePath.toString()}/${backupPath.toString()}`);
          return EMPTY;
        }),
        concatMap((files) => from(files)),
        filter((file) => FileBrowserService.isFileAuthorized(joinBuffer(backupPath, file), includes, excludes)),
        concatMap((file) =>
          from(this.createManifestFromLocalFile(sharePath, joinBuffer(backupPath, file))).pipe(
            catchError((err) => {
              this.logger.warn(`Can't read information of the file ${sharePath.toString()}/${backupPath.toString()}`);
              return EMPTY;
            }),
          ),
        ),
      );
      const folders$ = files$.pipe(
        filter((folder) => FileBrowserService.isDirectory(folder.stats?.mode || Long.ZERO)),
        concatMap((folder) => forShare(folder.path, includes, excludes)),
      );
      return merge(files$, folders$);
    };
    return forShare;
  }

  private async createManifestFromLocalFile(sharePath: Buffer, path: Buffer): Promise<FileManifest> {
    const fileStat = await lstat(joinBuffer(sharePath, path), { bigint: true });
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

  public static isRegularFile(mode: Long = Long.ZERO): boolean {
    return (mode.toNumber() & constantsFs.S_IFMT) === constantsFs.S_IFREG;
  }

  public static isDirectory(mode: Long = Long.ZERO): boolean {
    return (mode.toNumber() & constantsFs.S_IFMT) === constantsFs.S_IFDIR;
  }
}
