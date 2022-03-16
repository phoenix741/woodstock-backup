import { Injectable } from '@nestjs/common';
import { FileBrowserService, FileManifest, ManifestService, SEPARATOR, splitBuffer, unmangle } from '@woodstock/shared';
import { AsyncIterableX } from 'ix/asynciterable';
import { filter, map } from 'ix/asynciterable/operators';
import { BackupsService } from './backups.service';

@Injectable()
export class FilesService {
  constructor(
    private backupService: BackupsService,
    private fileBrowserService: FileBrowserService,
    private manifestService: ManifestService,
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

  searchFiles(hostname: string, backupNumber: number, sharePath: Buffer, path?: Buffer): AsyncIterableX<FileManifest> {
    const manifest = this.backupService.getManifest(hostname, backupNumber, sharePath);
    const searchPath = splitBuffer(path || Buffer.alloc(0));

    return this.manifestService.readManifestEntries(manifest).pipe(
      map((manifest) => ({
        ...manifest,
        path: splitBuffer(manifest.path),
      })),
      filter((manifest) => this.filterPath(manifest.path, searchPath)),
      map((manifest) => ({
        ...manifest,
        path: manifest.path[manifest.path.length - 1],
      })),
    );
  }

  private filterPath(path: Buffer[], searchPath: Buffer[]) {
    if (path.length !== searchPath.length + 1) {
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
