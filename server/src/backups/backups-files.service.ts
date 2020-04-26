import { BadRequestException, Injectable, NotFoundException } from '@nestjs/common';
import * as filetype from 'file-type';
import * as fs from 'fs';
import { isAbsolute, join } from 'path';

import { EnumFileType, FileDescription } from './backups-files.dto';
import { BackupsService } from './backups.service';

@Injectable()
export class BackupsFilesService {
  constructor(private backupsService: BackupsService) {}

  async list(name: string, number: number, path = '/'): Promise<FileDescription[]> {
    if (!isAbsolute(path)) {
      throw new BadRequestException('Only absolute path can be used to serach for directory');
    }

    try {
      const destinationDirectory = this.backupsService.getDestinationDirectory(name, number);
      const files = await fs.promises.readdir(join(destinationDirectory, path), { withFileTypes: true });
      return Promise.all(
        files.map(async file => ({
          name: file.name,
          type: this.getFileType(file),
          ...(await this.getFileStat(join(destinationDirectory, path, file.name))),
        })),
      );
    } catch (err) {
      throw new NotFoundException(err);
    }
  }

  async getFileName(
    name: string,
    number: number,
    path: string,
  ): Promise<{ filename: string; mimetype?: string; stats: fs.Stats }> {
    if (!isAbsolute(path)) {
      throw new BadRequestException('Only absolute path can be used to serach for directory');
    }

    try {
      const destinationDirectory = this.backupsService.getDestinationDirectory(name, number);
      const filename = join(destinationDirectory, path);
      const stats = await this.getFileStat(filename);
      const mimetype = (stats.isFile() && (await filetype.fromFile(filename))?.mime) || undefined;

      return {
        filename,
        stats,
        mimetype,
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
