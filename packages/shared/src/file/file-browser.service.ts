import { Injectable, Logger, UnauthorizedException } from '@nestjs/common';
import { constants as constantsFs } from 'fs';
import { lstat, readdir, access } from 'fs/promises';
import { AsyncIterableX, from, merge, of } from 'ix/asynciterable';
import { catchError, filter, flatMap, map } from 'ix/asynciterable/operators';
import * as Long from 'long';
import { FileManifest } from '../models/woodstock';
import { notUndefined } from '../utils/iterator.utils';
import { bigIntToLong } from '../utils/number.utils';
import { joinBuffer } from '../utils/path.utils';

@Injectable()
export class FileBrowserService {
  private logger = new Logger(FileBrowserService.name);

  public getFiles(
    sharePath: Buffer,
  ): (backupPath: Buffer, includes: RegExp[], excludes: RegExp[]) => AsyncIterableX<FileManifest> {
    const forShare = (
      backupPath: Buffer,
      includes: RegExp[] = [],
      excludes: RegExp[] = [],
    ): AsyncIterableX<FileManifest> => {
      const files$ = from(readdir(joinBuffer(sharePath, backupPath), { encoding: 'buffer' })).pipe(
        catchError<Buffer[], Buffer[]>(() => {
          this.logger.warn(`Can't read the directory ${sharePath.toString()}/${backupPath.toString()}`);
          return of([]);
        }),
        flatMap((files) => from(files)),
        filter((file) => FileBrowserService.isFileAuthorized(joinBuffer(backupPath, file), includes, excludes)),
        map<Buffer, FileManifest | undefined>(async (file) => {
          try {
            return await this.createManifestFromLocalFile(sharePath, joinBuffer(backupPath, file));
          } catch (err) {
            this.logger.warn(
              `Can't read information of the file ${sharePath.toString()}/${backupPath.toString()}: ${
                err instanceof Error ? err.message : err
              }`,
            );
            return undefined;
          }
        }),
        notUndefined(),
      );

      const folders$ = files$.pipe(
        filter((folder) => FileBrowserService.isDirectory(folder.stats?.mode || Long.ZERO)),
        flatMap((folder) => forShare(folder.path, includes, excludes)),
      );

      return merge(files$, folders$);
    };
    return forShare;
  }

  private async createManifestFromLocalFile(sharePath: Buffer, path: Buffer): Promise<FileManifest> {
    const [fileStat, fileAccess] = await Promise.all([
      lstat(joinBuffer(sharePath, path), { bigint: true }),
      access(joinBuffer(sharePath, path))
        .then(() => true)
        .catch(() => false),
    ]);
    if (!fileAccess) {
      throw new UnauthorizedException(`Can't read the file ${sharePath.toString()}/${path.toString()}`);
    }

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
      xattr: {},
      acl: [],
      chunks: [],
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
