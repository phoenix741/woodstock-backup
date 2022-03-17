import { Injectable } from '@nestjs/common';
import {
  FileBrowserService,
  FileManifest,
  longToBigInt,
  ManifestService,
  splitBuffer,
  unmangle,
} from '@woodstock/shared';
import { Archiver } from 'archiver';
import { Stats } from 'fs';
import { AsyncIterableX } from 'ix/asynciterable';
import { filter, map } from 'ix/asynciterable/operators';
import * as Long from 'long';
import * as MultiStream from 'multistream';
import { FactoryStream } from 'multistream';
import { Readable } from 'stream';
import { BackupsService } from './backups.service';
import { PoolService } from './pool/pool.service';

@Injectable()
export class FilesService {
  constructor(
    private backupService: BackupsService,
    private fileBrowserService: FileBrowserService,
    private manifestService: ManifestService,
    private poolService: PoolService,
  ) {}

  /**
   * List all file in the backup directory of the hostname that represent a share path
   * @param hostname The host name
   * @param backupNumber The backup number
   */
  listShares(hostname: string, backupNumber: number): AsyncIterableX<Buffer> {
    const destinationDirectory = this.backupService.getDestinationDirectory(hostname, backupNumber);

    return this.fileBrowserService.getFilesFromDirectory(Buffer.from(destinationDirectory)).pipe(
      map((file) => (file.name as unknown as Buffer).toString('latin1')),
      filter((file) => file.endsWith('.manifest')),
      map((file) => unmangle(file.slice(0, -'.manifest'.length))),
    );
  }

  searchFiles(
    hostname: string,
    backupNumber: number,
    sharePath: Buffer,
    path?: Buffer,
    recursive = false,
  ): AsyncIterableX<FileManifest> {
    const manifest = this.backupService.getManifest(hostname, backupNumber, sharePath);
    const searchPath = splitBuffer(path || Buffer.alloc(0));

    return this.manifestService.readManifestEntries(manifest).pipe(
      map((manifest) => ({
        ...manifest,
        splittedPath: splitBuffer(manifest.path),
      })),
      filter((manifest) => this.filterPath(manifest.splittedPath, searchPath, recursive)),
      // map((manifest) => ({
      //   ...manifest,
      // })),
    );
  }

  /**
   * Create a stream that can be used to read the file describe by the manifest
   * @param manifest The file manifest
   */
  readFileStream(manifest: FileManifest): Readable {
    if (FileBrowserService.isSpecialFile(longToBigInt(manifest.stats?.mode || Long.ZERO))) {
      return Readable.from([]);
    }

    let currentChunk = 0;
    const chunkReadableFactory: FactoryStream = (cb) => {
      if (!manifest.chunks || currentChunk >= manifest.chunks.length) {
        return cb(null, null);
      }
      setImmediate(() => {
        const wrapper = this.poolService.getChunk(manifest.chunks[currentChunk++]);
        cb(null, wrapper.read());
      });
    };
    return new MultiStream(chunkReadableFactory);
  }

  async createArchive(archiver: Archiver, hostname: string, backupNumber: number, sharePath: Buffer, path?: Buffer) {
    console.log('createArchive', hostname, backupNumber, sharePath.toString('utf-8'), path?.toString('utf-8'));
    const manifests = this.searchFiles(hostname, backupNumber, sharePath, path, true);
    for await (const manifest of manifests) {
      const mode = longToBigInt(manifest.stats?.mode || Long.ZERO);
      console.log('manifest', manifest, mode);

      if (FileBrowserService.isRegularFile(mode)) {
        archiver.append(this.readFileStream(manifest), {
          name: manifest.path.toString('utf-8'),
          date: manifest.stats?.lastModified && new Date(manifest.stats?.lastModified.toNumber()),
          mode: manifest.stats?.mode?.toNumber(),
          stats: {
            size: manifest.stats?.size?.toNumber(),
            mode: Number(mode),
            mtime: manifest.stats?.lastModified && new Date(manifest.stats?.lastModified.toNumber()),
            isFile: () => FileBrowserService.isRegularFile(mode),
            isDirectory: () => FileBrowserService.isDirectory(mode),
            isSymbolicLink: () => FileBrowserService.isSymLink(mode),

            dev: manifest.stats?.dev?.toNumber(),
            ino: manifest.stats?.ino?.toNumber(),
            nlink: manifest.stats?.nlink?.toNumber(),
            uid: manifest.stats?.ownerId?.toNumber(),
            gid: manifest.stats?.groupId?.toNumber(),
            rdev: manifest.stats?.rdev?.toNumber(),
            atime: manifest.stats?.lastRead && new Date(manifest.stats?.lastRead.toNumber()),
            ctime: manifest.stats?.created && new Date(manifest.stats?.created.toNumber()),
          } as Stats,
        });
      } else if (FileBrowserService.isSymLink(mode) && manifest.symlink) {
        archiver.symlink(manifest.path.toString('utf-8'), manifest.symlink?.toString('utf-8'), Number(mode));
      }
    }
  }

  private filterPath(path: Buffer[], searchPath: Buffer[], recursive = false) {
    if (recursive ? path.length <= searchPath.length : path.length !== searchPath.length + 1) {
      return false;
    }

    for (let i = 0; i < searchPath.length; i++) {
      if (!path[i].equals(searchPath[i])) {
        return false;
      }
    }
    return true;
  }
}
