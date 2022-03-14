import { BadRequestException, Injectable, NotFoundException } from '@nestjs/common';
import { BackupsService, EnumFileType, FileDescription } from '@woodstock/backoffice-shared';
import { FilesService } from '@woodstock/backoffice-shared/services/files.service';
import { mangle } from '@woodstock/shared';
import * as fs from 'fs';
import { toArray } from 'ix/asynciterable';
import { map } from 'ix/asynciterable/operators';
import { isAbsolute, join } from 'path';

@Injectable()
export class BackupsFilesService {
  constructor(private backupsService: BackupsService, private filesService: FilesService) {}

  async listShare(name: string, number: number): Promise<FileDescription[]> {
    try {
      const shares = this.filesService.listShares(name, number).pipe(
        map((name) => ({
          name,
          type: EnumFileType.SHARE,

          dev: -1,
          ino: -1,
          mode: -1,
          nlink: -1,
          uid: -1,
          gid: -1,
          rdev: -1,
          size: -1,
          blksize: -1,
          blocks: -1,

          atimeMs: -1,
          mtimeMs: -1,
          ctimeMs: -1,
          birthtimeMs: -1,

          atime: new Date(),
          mtime: new Date(),
          ctime: new Date(),
          birthtime: new Date(),
        })),
      );

      return await toArray(shares);
    } catch (err) {
      throw new NotFoundException(err);
    }
  }

  async list(name: string, number: number, sharePath: string, path = '/'): Promise<FileDescription[]> {
    if (!isAbsolute(sharePath) || !isAbsolute(path)) {
      throw new BadRequestException('Only absolute path can be used to serach for directory');
    }

    try {
      const destinationDirectory = join(
        this.backupsService.getDestinationDirectory(name, number),
        mangle(sharePath),
        path,
      );

      const files = await fs.promises.readdir(destinationDirectory, { withFileTypes: true, encoding: 'binary' });
      return Promise.all(
        files.map(async (file) => ({
          name: mangle(file.name),
          type: this.getFileType(file),
          ...(await this.getFileStat(join(destinationDirectory, file.name))),
        })),
      );
    } catch (err) {
      console.log(err.stack);
      throw new NotFoundException(err);
    }
  }

  async getFileName(
    name: string,
    number: number,
    sharePath: string,
    path: string,
  ): Promise<{ filename: string; stats: fs.Stats }> {
    if (!isAbsolute(sharePath) || !isAbsolute(path)) {
      throw new BadRequestException('Only absolute path can be used to serach for directory');
    }

    try {
      const filename = join(this.backupsService.getDestinationDirectory(name, number), mangle(sharePath), path);
      const stats = await this.getFileStat(filename);

      return {
        filename,
        stats,
      };
    } catch (err) {
      throw new NotFoundException(err);
    }
  }

  private getFileType(file: fs.Dirent): EnumFileType {
    if (file.isBlockDevice()) {
      return EnumFileType.BLOCK_DEVICE;
    }
    if (file.isCharacterDevice()) {
      return EnumFileType.CHARACTER_DEVICE;
    }
    if (file.isDirectory()) {
      return EnumFileType.DIRECTORY;
    }
    if (file.isFIFO()) {
      return EnumFileType.FIFO;
    }
    if (file.isFile()) {
      return EnumFileType.REGULAR_FILE;
    }
    if (file.isSocket()) {
      return EnumFileType.SOCKET;
    }
    if (file.isSymbolicLink()) {
      return EnumFileType.SYMBOLIC_LINK;
    }
    return EnumFileType.UNKNOWN;
  }

  private async getFileStat(file: string) {
    return await fs.promises.lstat(file);
  }
}
