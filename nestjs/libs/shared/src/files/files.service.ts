import { Injectable } from '@nestjs/common';
import { CoreBackupsService, CoreFilesService, JsFileManifest, JsFileManifestType } from '@woodstock/shared-rs';
import { Archiver } from 'archiver';
import { Stats } from 'fs';
import { AsyncIterableX, AsyncSink, create } from 'ix/asynciterable';
import { Readable } from 'stream';

@Injectable()
export class FilesService {
  constructor(
    private backupService: CoreBackupsService,
    private fileService: CoreFilesService,
  ) {}

  /**
   * List all file in the backup directory of the hostname that represent a share path
   * @param hostname The host name
   * @param backupNumber The backup number
   */
  listShares(hostname: string, backupNumber: number): Promise<string[]> {
    return this.backupService.getBackupSharePaths(hostname, backupNumber);
  }

  searchFiles(
    hostname: string,
    backupNumber: number,
    sharePath: string,
    path?: Buffer,
    recursive = false,
  ): AsyncIterableX<JsFileManifest> {
    const sink = new AsyncSink<JsFileManifest>();

    this.fileService.list(hostname, backupNumber, sharePath, path ?? Buffer.from(''), recursive, (err, manifest) => {
      if (manifest) {
        sink.write(manifest);
      } else {
        sink.end();
      }
    });

    return create(() => sink[Symbol.asyncIterator]());
  }

  /**
   * Create a stream that can be used to read the file describe by the manifest
   * @param manifest The file manifest
   */
  readFileStream(manifest: JsFileManifest): Readable {
    const sink = new AsyncSink<Buffer>();

    this.fileService.readManifest(manifest, (err, data) => {
      if (data) {
        sink.write(data);
      } else {
        sink.end();
      }
    });

    return Readable.from(create(() => sink[Symbol.asyncIterator]()));
  }

  async createArchive(archiver: Archiver, hostname: string, backupNumber: number, sharePath: string, path?: Buffer) {
    const manifests = this.searchFiles(hostname, backupNumber, sharePath, path, true);

    for await (const manifest of manifests) {
      const isRegular = manifest.stats?.type === JsFileManifestType.RegularFile;
      const isDirectory = manifest.stats?.type === JsFileManifestType.Directory;
      const isSymLink = manifest.stats?.type === JsFileManifestType.Symlink;

      if (isRegular) {
        archiver.append(this.readFileStream(manifest), {
          name: manifest.path.toString('utf-8'),
          date: manifest.stats?.lastModified ? new Date(manifest.stats.lastModified) : undefined,
          mode: manifest.stats?.mode,
          stats: {
            size: manifest.stats?.size ? Number(manifest.stats?.size) : 0,
            mode: manifest.stats?.mode,
            mtime: manifest.stats?.lastModified ? new Date(manifest.stats?.lastModified) : undefined,
            isFile: () => isRegular,
            isDirectory: () => isDirectory,
            isSymbolicLink: () => isSymLink,
            isBlockDevice: () => manifest.stats?.type === JsFileManifestType.BlockDevice,
            isCharacterDevice: () => manifest.stats?.type === JsFileManifestType.CharacterDevice,
            isFIFO: () => manifest.stats?.type === JsFileManifestType.Fifo,
            isSocket: () => manifest.stats?.type === JsFileManifestType.Socket,

            dev: manifest.stats?.dev ? Number(manifest.stats?.dev) : 0,
            ino: manifest.stats?.ino ? Number(manifest.stats?.ino) : 0,
            nlink: manifest.stats?.nlink ? Number(manifest.stats?.nlink) : 0,
            uid: manifest.stats?.ownerId,
            gid: manifest.stats?.groupId,
            rdev: manifest.stats?.rdev ? Number(manifest.stats?.rdev) : 0,
            atime: manifest.stats?.lastRead ? new Date(manifest.stats?.lastRead) : undefined,
            ctime: manifest.stats?.created ? new Date(manifest.stats?.created) : undefined,

            blksize: 0,
            blocks: 0,
            atimeMs: 0,
            mtimeMs: 0,
          } as Stats,
        });
      } else if (isSymLink && manifest.symlink) {
        archiver.symlink(manifest.path.toString('utf-8'), manifest.symlink?.toString('utf-8'), manifest.stats?.mode);
      }
    }
  }
}
