import { BadRequestException, Injectable, NotFoundException } from '@nestjs/common';
import { BackupsService, EnumFileType, FileDescription } from '@woodstock/backoffice-shared';
import { FilesService } from '@woodstock/backoffice-shared/services/files.service';
import { mangle, unmangle } from '@woodstock/shared';
import * as fs from 'fs';
import { toArray } from 'ix/asynciterable';
import { map } from 'ix/asynciterable/operators';
import { isAbsolute, join } from 'path';

@Injectable()
export class BackupsFilesService {
  constructor(private backupsService: BackupsService, private filesService: FilesService) {}

  async listShare(name: string, number: number): Promise<FileDescription[]> {
    const backup = await this.backupsService.getBackup(name, number);
    const startDate = backup.startDate;
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
          size: 0,
          blksize: 0,
          blocks: -1,

          atimeMs: startDate * 1000,
          mtimeMs: startDate * 1000,
          ctimeMs: startDate * 1000,
          birthtimeMs: startDate * 1000,

          atime: new Date(startDate),
          mtime: new Date(startDate),
          ctime: new Date(startDate),
          birthtime: new Date(startDate),
        })),
      );

      return await toArray(shares);
    } catch (err) {
      throw new NotFoundException(err);
    }
  }

  async list(name: string, number: number, sharePath: string, path = '/'): Promise<FileDescription[]> {
    try {
      return await toArray(
        this.filesService.listFiles(name, number, unmangle(sharePath), unmangle(path)).pipe(
          map((file) => ({
            name: mangle(file.path),
            type: EnumFileType.REGULAR_FILE,
            mode: file.stats.mode.toNumber(),
          })),
        ),
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
    /*
    try {
      const filename = join(this.backupsService.getDestinationDirectory(name, number), mangle(sharePath), path);
      const stats = await this.getFileStat(filename);

      return {
        filename,
        stats,
      };
    } catch (err) {*/
    throw new NotFoundException();
    // }
  }
}
