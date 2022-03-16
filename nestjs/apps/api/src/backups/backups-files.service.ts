import { BadRequestException, Injectable, NotFoundException } from '@nestjs/common';
import { BackupsService, FileDescription } from '@woodstock/backoffice-shared';
import { FilesService } from '@woodstock/backoffice-shared/services/files.service';
import { unmangle } from '@woodstock/shared';
import * as fs from 'fs';
import { toArray } from 'ix/asynciterable';
import { map } from 'ix/asynciterable/operators';
import * as Long from 'long';
import { isAbsolute } from 'path';

@Injectable()
export class BackupsFilesService {
  constructor(private backupsService: BackupsService, private filesService: FilesService) {}

  async listShare(name: string, number: number): Promise<FileDescription[]> {
    const backup = await this.backupsService.getBackup(name, number);
    const startDate = backup.startDate;
    const shares = this.filesService.listShares(name, number).pipe(
      map(
        (path) =>
          new FileDescription({
            path,
            stats: {
              mode: Long.fromNumber(-1),
              created: Long.fromNumber(startDate),
              lastRead: Long.fromNumber(startDate),
              lastModified: Long.fromNumber(startDate),
            },
          }),
      ),
    );

    return await toArray(shares);
  }

  async list(name: string, number: number, sharePath: string, path = '/'): Promise<FileDescription[]> {
    try {
      return (
        await toArray(
          this.filesService
            .searchFiles(name, number, unmangle(sharePath), unmangle(path))
            .pipe(map((file) => new FileDescription(file))),
        )
      ).sort((a, b) => a.type.localeCompare(b.type) || a.path.compare(b.path));
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
