import { BadRequestException, Injectable, NotFoundException } from '@nestjs/common';
import { mangle, unmangle } from '@woodstock/shared';
import * as fs from 'fs';
import { isAbsolute, join } from 'path';

import { EnumFileType, FileDescription } from './backups-files.dto';
import { BackupsService } from './backups.service';

@Injectable()
export class BackupsFilesService {
  constructor(private backupsService: BackupsService) {}

  async listShare(name: string, number: number): Promise<FileDescription[]> {
    try {
      const destinationDirectory = this.backupsService.getDestinationDirectory(name, number);

      const files = await fs.promises.readdir(destinationDirectory, { withFileTypes: true });

      return Promise.all(
        files.map(async (file) => ({
          name: unmangle(file.name),
          type: this.getFileType(file),
          ...(await this.getFileStat(join(destinationDirectory, file.name))),
        })),
      );
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
          name: Buffer.from(file.name),
          type: this.getFileType(file),
          ...(await this.getFileStat(join(destinationDirectory, file.name))),
        })),
      );
    } catch (err) {
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
