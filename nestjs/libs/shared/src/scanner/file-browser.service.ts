import { Injectable, Logger, UnauthorizedException } from '@nestjs/common';
import { constants as constantsFs, Dirent } from 'fs';
import { access, lstat, opendir, readlink } from 'fs/promises';
import { AsyncIterableX, from, of } from 'ix/asynciterable';
import { concatMap, filter, map, startWith } from 'ix/asynciterable/operators';
import type { IMinimatch } from 'minimatch';
import { FileManifest } from '../shared';
import { joinBuffer, bigIntToLong, notUndefined } from '../utils';

function isFileAuthorized(file: Buffer, includes: IMinimatch[], excludes: IMinimatch[]): boolean {
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

@Injectable()
export class FileBrowserService {
  #logger = new Logger(FileBrowserService.name);

  #readDirSync(path: Buffer): AsyncIterableX<Dirent> {
    return from(
      opendir(path, { encoding: 'buffer' as any }).catch((err) => {
        this.#logger.error(err.message);
        return from([]);
      }),
    ).pipe(concatMap((dir) => dir));
  }

  getFilesFromDirectory(
    sharePath: Buffer,
    backupPath?: Buffer,
    filterCallback?: (sharePath: Buffer, backupPath: Buffer | undefined, path: Dirent) => boolean,
  ): AsyncIterableX<Dirent> {
    const path = backupPath ? joinBuffer(sharePath, backupPath) : sharePath;
    return this.#readDirSync(path).pipe(
      filter((dirEntry) => !filterCallback || filterCallback(sharePath, backupPath, dirEntry)),
    );
  }

  getFilesRecursive(
    sharePath: Buffer,
    filterCallback?: (sharePath: Buffer, backupPath: Buffer | undefined, path: Dirent) => boolean,
  ): (backupPath: Buffer) => AsyncIterableX<Buffer> {
    const forShare = (backupPath: Buffer): AsyncIterableX<Buffer> => {
      const files = this.getFilesFromDirectory(sharePath, backupPath, filterCallback).pipe(
        concatMap<Dirent, Buffer>(async (dirEntry) => {
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

  getFiles(
    sharePath: Buffer,
  ): (backupPath: Buffer, includes: IMinimatch[], excludes: IMinimatch[]) => AsyncIterableX<FileManifest> {
    const forShare = (
      backupPath: Buffer,
      includes: IMinimatch[] = [],
      excludes: IMinimatch[] = [],
    ): AsyncIterableX<FileManifest> => {
      return this.getFilesRecursive(joinBuffer(sharePath, backupPath), (sharePath, backupPath, path) =>
        isFileAuthorized(joinBuffer(backupPath ?? sharePath, path.name as unknown as Buffer), includes, excludes),
      )(backupPath).pipe(
        map<Buffer, FileManifest | undefined>(async (file) => {
          try {
            return await this.#createManifestFromLocalFile(sharePath, file);
          } catch (err) {
            this.#logger.warn(
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

  async #createManifestFromLocalFile(sharePath: Buffer, path: Buffer): Promise<FileManifest> {
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

  static isRegularFile(mode: bigint): boolean {
    return (Number(mode) & constantsFs.S_IFMT) === constantsFs.S_IFREG;
  }

  static isSymLink(mode: bigint): boolean {
    return (Number(mode) & constantsFs.S_IFMT) === constantsFs.S_IFLNK;
  }

  static isDirectory(mode: bigint): boolean {
    return (Number(mode) & constantsFs.S_IFMT) === constantsFs.S_IFDIR;
  }

  static isBlockDevice(mode: bigint): boolean {
    return (Number(mode) & constantsFs.S_IFMT) === constantsFs.S_IFBLK;
  }

  static isCharacterDevice(mode: bigint): boolean {
    return (Number(mode) & constantsFs.S_IFMT) === constantsFs.S_IFCHR;
  }

  static isFIFO(mode: bigint): boolean {
    return (Number(mode) & constantsFs.S_IFMT) === constantsFs.S_IFIFO;
  }

  static isSocket(mode: bigint): boolean {
    return (Number(mode) & constantsFs.S_IFMT) === constantsFs.S_IFSOCK;
  }

  static isSpecialFile(mode: bigint): boolean {
    return (Number(mode) & constantsFs.S_IFMT) !== constantsFs.S_IFREG;
  }
}
