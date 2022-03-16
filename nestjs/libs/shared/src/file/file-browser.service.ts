import { Injectable, Logger, UnauthorizedException } from '@nestjs/common';
import { constants as constantsFs, Dirent } from 'fs';
import { access, lstat, opendir, readlink } from 'fs/promises';
import { AsyncIterableX, from, of, pipe } from 'ix/asynciterable';
import { filter, flatMap, map, startWith } from 'ix/asynciterable/operators';
import { IMinimatch } from 'minimatch';
import { FileManifest } from '../models/woodstock';
import { notUndefined } from '../utils/iterator.utils';
import { bigIntToLong } from '../utils/number.utils';
import { joinBuffer } from '../utils/path.utils';

@Injectable()
export class FileBrowserService {
  private logger = new Logger(FileBrowserService.name);

  public getFilesFromDirectory(
    path: Buffer,
    filterCallback?: (currentPath: Buffer, path: Dirent) => boolean,
  ): AsyncIterableX<Dirent> {
    return pipe(
      from(
        opendir(path, { encoding: 'buffer' as any }).catch((err) => {
          this.logger.error(err);
          return from([] as Dirent[]);
        }),
      ),
      flatMap((dir) => dir),
      filter((dirEntry) => !filterCallback || filterCallback(path, dirEntry)),
    );
  }

  public getFilesRecursive(
    sharePath: Buffer,
    filterCallback?: (currentPath: Buffer, path: Dirent) => boolean,
  ): (backupPath: Buffer) => AsyncIterableX<Buffer> {
    const forShare = (backupPath: Buffer): AsyncIterableX<Buffer> => {
      const path = joinBuffer(sharePath, backupPath);
      const files = this.getFilesFromDirectory(path, filterCallback).pipe(
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
  ): (backupPath: Buffer, includes: IMinimatch[], excludes: IMinimatch[]) => AsyncIterableX<FileManifest> {
    const forShare = (
      backupPath: Buffer,
      includes: IMinimatch[] = [],
      excludes: IMinimatch[] = [],
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
    const file = joinBuffer(sharePath, path);

    const [fileStat, fileAccess] = await Promise.all([
      lstat(file, { bigint: true }),
      access(file, constantsFs.R_OK)
        .then(() => true)
        .catch(() => false),
    ]);
    if (
      (FileBrowserService.isDirectory(fileStat.mode) || FileBrowserService.isRegularFile(fileStat.mode)) &&
      !fileAccess
    ) {
      throw new UnauthorizedException(`The file is not readable by current user`);
    }

    let symlink: Buffer | undefined;
    if (FileBrowserService.isSymLink(fileStat.mode)) {
      symlink = await readlink(file, { encoding: 'buffer' });
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
        dev: bigIntToLong(fileStat.dev),
        rdev: bigIntToLong(fileStat.rdev),
        ino: bigIntToLong(fileStat.ino),
        nlink: bigIntToLong(fileStat.nlink),
      },
      xattr: {},
      acl: [],
      chunks: [],
      symlink,
    };
  }

  private static isFileAuthorized(file: Buffer, includes: IMinimatch[], excludes: IMinimatch[]): boolean {
    const latin1File = file.toString('latin1');
    if (includes.length > 0) {
      if (!includes.some((include) => include.match(latin1File))) {
        return false;
      }
    }

    if (excludes.length > 0) {
      if (excludes.some((exclude) => exclude.match(latin1File))) {
        return false;
      }
    }

    return true;
  }

  public static isRegularFile(mode: bigint): boolean {
    return (Number(mode) & constantsFs.S_IFMT) === constantsFs.S_IFREG;
  }

  public static isSymLink(mode: bigint): boolean {
    return (Number(mode) & constantsFs.S_IFMT) === constantsFs.S_IFLNK;
  }

  public static isDirectory(mode: bigint): boolean {
    return (Number(mode) & constantsFs.S_IFMT) === constantsFs.S_IFDIR;
  }

  public static isBlockDevice(mode: bigint): boolean {
    return (Number(mode) & constantsFs.S_IFMT) === constantsFs.S_IFBLK;
  }

  public static isCharacterDevice(mode: bigint): boolean {
    return (Number(mode) & constantsFs.S_IFMT) === constantsFs.S_IFCHR;
  }

  public static isFIFO(mode: bigint): boolean {
    return (Number(mode) & constantsFs.S_IFMT) === constantsFs.S_IFIFO;
  }

  public static isSocket(mode: bigint): boolean {
    return (Number(mode) & constantsFs.S_IFMT) === constantsFs.S_IFSOCK;
  }

  public static isSpecialFile(mode: bigint): boolean {
    return (Number(mode) & constantsFs.S_IFMT) !== constantsFs.S_IFREG;
  }
}
