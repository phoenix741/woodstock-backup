import { Injectable, Logger, UnauthorizedException } from '@nestjs/common';
import { constants as constantsFs, Dirent } from 'fs';
import { access, lstat, opendir } from 'fs/promises';
import { AsyncIterableX, from, of, pipe } from 'ix/asynciterable';
import { filter, flatMap, map, startWith } from 'ix/asynciterable/operators';
import * as Long from 'long';
import { FileManifest } from '../models/woodstock';
import { notUndefined } from '../utils/iterator.utils';
import { bigIntToLong } from '../utils/number.utils';
import { joinBuffer } from '../utils/path.utils';

@Injectable()
export class FileBrowserService {
  private logger = new Logger(FileBrowserService.name);

  public getFilesRecursive(
    sharePath: Buffer,
    filterCallback?: (currentPath: Buffer, path: Dirent) => boolean,
  ): (backupPath: Buffer) => AsyncIterableX<Buffer> {
    const forShare = (backupPath: Buffer): AsyncIterableX<Buffer> => {
      const path = joinBuffer(sharePath, backupPath);
      const files = pipe(
        from(
          opendir(path, { encoding: 'buffer' as any }).catch((err) => {
            this.logger.error(err);
            return from([] as Dirent[]);
          }),
        ),
        flatMap((dir) => dir),
        filter((dirEntry) => !filterCallback || filterCallback(path, dirEntry)),
        flatMap(async (dirEntry) => {
          if (dirEntry.isDirectory()) {
            return forShare(joinBuffer(backupPath, dirEntry.name as unknown as Buffer)).pipe(
              startWith(joinBuffer(backupPath, dirEntry.name as unknown as Buffer)),
            );
          }
          return of(joinBuffer(backupPath, dirEntry.name as unknown as Buffer));
        }),
      );
      return files;
    };

    return forShare;
  }

  public getFiles(
    sharePath: Buffer,
  ): (backupPath: Buffer, includes: RegExp[], excludes: RegExp[]) => AsyncIterableX<FileManifest> {
    const forShare = (
      backupPath: Buffer,
      includes: RegExp[] = [],
      excludes: RegExp[] = [],
    ): AsyncIterableX<FileManifest> => {
      return this.getFilesRecursive(joinBuffer(sharePath, backupPath), (currentPath, path) =>
        FileBrowserService.isFileAuthorized(
          joinBuffer(currentPath, path.name as unknown as Buffer),
          includes,
          excludes,
        ),
      )(backupPath).pipe(
        map<Buffer, FileManifest | undefined>(async (file) => {
          try {
            return await this.createManifestFromLocalFile(sharePath, file);
          } catch (err) {
            this.logger.warn(
              `Can't read information of the file ${joinBuffer(sharePath, file).toString()}: ${
                err instanceof Error ? err.message : err
              }`,
            );
            return undefined;
          }
        }),
        notUndefined(),
      );
    };
    return forShare;
  }

  private async createManifestFromLocalFile(sharePath: Buffer, path: Buffer): Promise<FileManifest> {
    const [fileStat, fileAccess] = await Promise.all([
      lstat(joinBuffer(sharePath, path), { bigint: true }),
      access(joinBuffer(sharePath, path), constantsFs.R_OK)
        .then(() => true)
        .catch(() => false),
    ]);
    if (!fileAccess) {
      throw new UnauthorizedException(`The file is not readable by current user`);
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
