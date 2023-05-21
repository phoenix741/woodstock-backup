import { Injectable, Logger, UnauthorizedException } from '@nestjs/common';
import { bigIntToLong, joinBuffer, notUndefined } from '@woodstock/core';
import { constants as constantsFs, Dirent } from 'fs';
import { access, lstat, opendir, readlink } from 'fs/promises';
import { AsyncIterableX, from, of } from 'ix/asynciterable';
import { concatMap, filter, map, startWith } from 'ix/asynciterable/operators';
import { Minimatch } from 'minimatch';
import { FileManifest } from '../protobuf';

function isFileAuthorized(file: Buffer, includes: Minimatch[], excludes: Minimatch[]): boolean {
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
  ): (backupPath: Buffer, includes: Minimatch[], excludes: Minimatch[]) => AsyncIterableX<FileManifest> {
    const forShare = (
      backupPath: Buffer,
      includes: Minimatch[] = [],
      excludes: Minimatch[] = [],
    ): AsyncIterableX<FileManifest> => {
      return this.getFilesRecursive(joinBuffer(sharePath, backupPath), (sharePath, backupPath, path) =>
        isFileAuthorized(joinBuffer(backupPath ?? sharePath, path.name as unknown as Buffer), includes, excludes),
      )(backupPath).pipe(
        map<Buffer, FileManifest | undefined>(async (file) => {
          try {
            return await this.#createManifestFromLocalFile(sharePath, file);
          } catch (err) {
            this.#logger.warn(
              `Can't read information of the file ${joinBuffer(sharePath, file).toString()}: ${err instanceof Error ? err.message : err
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
    let mode = Number(fileStat.mode);

    if (
      (FileBrowserService.isDirectory(mode) || FileBrowserService.isRegularFile(mode)) &&
      !fileAccess
    ) {
      throw new UnauthorizedException(`The file is not readable by current user`);
    }

    let symlink: Buffer | undefined;
    if (FileBrowserService.isSymLink(mode)) {
      symlink = await readlink(file, { encoding: 'buffer' });
    }

    return {
      path,
      stats: {
        ownerId: Number(fileStat.uid),
        groupId: Number(fileStat.gid),
        size: bigIntToLong(fileStat.size),
        mode: Number(fileStat.mode),
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

  static isRegularFile(mode: number): boolean {
    return (mode & constantsFs.S_IFMT) === constantsFs.S_IFREG;
  }

  static isSymLink(mode: number): boolean {
    return (mode & constantsFs.S_IFMT) === constantsFs.S_IFLNK;
  }

  static isDirectory(mode: number): boolean {
    return (mode & constantsFs.S_IFMT) === constantsFs.S_IFDIR;
  }

  static isBlockDevice(mode: number): boolean {
    return (mode & constantsFs.S_IFMT) === constantsFs.S_IFBLK;
  }

  static isCharacterDevice(mode: number): boolean {
    return (mode & constantsFs.S_IFMT) === constantsFs.S_IFCHR;
  }

  static isFIFO(mode: number): boolean {
    return (mode & constantsFs.S_IFMT) === constantsFs.S_IFIFO;
  }

  static isSocket(mode: number): boolean {
    return (mode & constantsFs.S_IFMT) === constantsFs.S_IFSOCK;
  }

  static isSpecialFile(mode: number): boolean {
    return (mode & constantsFs.S_IFMT) !== constantsFs.S_IFREG;
  }
}
