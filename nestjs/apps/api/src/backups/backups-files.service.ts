import { Injectable, NotFoundException } from '@nestjs/common';
import { unmangle, FileDescription, BackupsService, FilesService } from '@woodstock/shared';
import { JsFileManifestType } from '@woodstock/shared-rs';
import { Archiver } from 'archiver';
import { toArray } from 'ix/asynciterable';
import { map } from 'ix/asynciterable/operators';

@Injectable()
export class BackupsFilesService {
  constructor(
    private backupsService: BackupsService,
    private filesService: FilesService,
  ) {}

  async listShare(name: string, number: number): Promise<FileDescription[]> {
    const backup = await this.backupsService.getBackup(name, number);
    if (!backup) {
      throw new NotFoundException();
    }

    const startDate = backup.startDate;
    const shares = (await this.filesService.listShares(name, number)).map(
      (path) =>
        new FileDescription({
          path: Buffer.from(path),
          stats: {
            mode: -1,
            created: startDate,
            lastRead: startDate,
            lastModified: startDate,
            ownerId: 0,
            groupId: 0,
            size: 0n,
            compressedSize: 0n,
            type: JsFileManifestType.Directory,
            dev: 0n,
            rdev: 0n,
            ino: 0n,
            nlink: 0n,
          },
          symlink: Buffer.from(''),
          acl: [],
          chunks: [],
          hash: Buffer.from(''),
          metadata: {},
          xattr: [],
        }),
    );
    return shares;
  }

  async list(name: string, number: number, sharePath: string, path = '/'): Promise<FileDescription[]> {
    return (
      await toArray(
        this.filesService
          .searchFiles(name, number, sharePath, unmangle(path))
          .pipe(map((file) => new FileDescription(file))),
      )
    ).sort((a, b) => a.type - b.type || a.path.compare(b.path));
  }

  async createArchive(archiver: Archiver, hostname: string, backupNumber: number, sharePath: string, path = '') {
    return this.filesService.createArchive(archiver, hostname, backupNumber, sharePath, unmangle(path));
  }
}
