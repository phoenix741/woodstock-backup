import { Injectable, NotFoundException } from '@nestjs/common';
import { unmangle } from '@woodstock/core';
import { BackupsService, FileDescription, FilesService } from '@woodstock/server';
import { Archiver } from 'archiver';
import { toArray } from 'ix/asynciterable';
import { map } from 'ix/asynciterable/operators';
import * as Long from 'long';

@Injectable()
export class BackupsFilesService {
  constructor(private backupsService: BackupsService, private filesService: FilesService) {}

  async listShare(name: string, number: number): Promise<FileDescription[]> {
    const backup = await this.backupsService.getBackup(name, number);
    if (!backup) {
      throw new NotFoundException();
    }

    const startDate = backup.startDate;
    const shares = this.filesService.listShares(name, number).pipe(
      map(
        (path) =>
          new FileDescription({
            path,
            stats: {
              mode: -1,
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
    return (
      await toArray(
        this.filesService
          .searchFiles(name, number, unmangle(sharePath), unmangle(path))
          .pipe(map((file) => new FileDescription(file))),
      )
    ).sort((a, b) => a.type.localeCompare(b.type) || a.path.compare(b.path));
  }

  async createArchive(archiver: Archiver, hostname: string, backupNumber: number, sharePath: string, path = '') {
    return this.filesService.createArchive(archiver, hostname, backupNumber, unmangle(sharePath), unmangle(path));
  }
}
