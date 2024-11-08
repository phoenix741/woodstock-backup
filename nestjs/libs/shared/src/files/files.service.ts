import { Injectable } from '@nestjs/common';
import {
  CoreBackupsService,
  CoreFilesService,
  JsFileManifest,
  JsFileManifestType,
  ViewerService,
} from '@woodstock/shared-rs';
import { Archiver } from 'archiver';
import { Stats } from 'fs';
import { Readable } from 'node:stream';
import { CallbackReadable } from './manifest.readable';
import * as TTLCache from '@isaacs/ttlcache';

const FILE_VIEW_MAX_ELEMENTS = process.env.FILE_VIEW_MAX_ELEMENTS ? parseInt(process.env.FILE_VIEW_MAX_ELEMENTS) : 10;
const FILE_VIEW_TTL_CACHE = process.env.FILE_VIEW_TTL_CACHE ? parseInt(process.env.FILE_VIEW_TTL_CACHE) : 15 * 60_000;

@Injectable()
export class FilesService {
  private cache = new TTLCache({ max: FILE_VIEW_MAX_ELEMENTS, ttl: FILE_VIEW_TTL_CACHE });

  constructor(
    private backupService: CoreBackupsService,
    private fileService: CoreFilesService,
  ) {}

  #getViewer(hostname: string, backupNumber: number): ViewerService {
    const key = `${hostname}-${backupNumber}`;
    const viewer = this.cache.get<ViewerService>(key) ?? this.fileService.createViewer(hostname, backupNumber);
    this.cache.set(key, viewer);
    return viewer;
  }

  /**
   * List all file in the backup directory of the hostname that represent a share path
   * @param hostname The host name
   * @param backupNumber The backup number
   */
  listShares(hostname: string, backupNumber: number): Promise<string[]> {
    return this.backupService.getBackupSharePaths(hostname, backupNumber);
  }

  async list(hostname: string, backupNumber: number, sharePath: string, path: Buffer): Promise<Array<JsFileManifest>> {
    const view = this.#getViewer(hostname, backupNumber);
    const files = await view.listDir(sharePath, path);
    return files;
  }

  /**
   * Create a stream that can be used to read the file describe by the manifest
   * @param manifest The file manifest
   */
  readFileStream(manifest: JsFileManifest): Readable {
    return new CallbackReadable(manifest, this.fileService);
  }

  async createArchive(archiver: Archiver, hostname: string, backupNumber: number, sharePath: string, path: Buffer) {
    const view = this.#getViewer(hostname, backupNumber);
    const manifests = await view.listDirRecursive(sharePath, path);

    for await (const manifest of manifests) {
      const isRegular = manifest.stats?.type === JsFileManifestType.RegularFile;
      const isDirectory = manifest.stats?.type === JsFileManifestType.Directory;
      const isSymLink = manifest.stats?.type === JsFileManifestType.Symlink;

      if (isRegular) {
        archiver.append(this.readFileStream(manifest), {
          name: manifest.path.toString('utf-8'),
          date: manifest.stats?.lastModified ? new Date(manifest.stats.lastModified * 1000) : undefined,
          mode: manifest.stats?.mode,
          stats: {
            size: manifest.stats?.size ? Number(manifest.stats?.size) : 0,
            mode: manifest.stats?.mode,
            mtime: manifest.stats?.lastModified ? new Date(manifest.stats?.lastModified * 1000) : undefined,
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
            atime: manifest.stats?.lastRead ? new Date(manifest.stats?.lastRead * 1000) : undefined,
            ctime: manifest.stats?.created ? new Date(manifest.stats?.created * 1000) : undefined,

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
