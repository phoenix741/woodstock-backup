import { Injectable } from '@nestjs/common';
import { FileBrowserService, FileManifest, ManifestService, SEPARATOR } from '@woodstock/shared';
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
  listShares(hostname: string, backupNumber: number): AsyncIterableX<string> {
    const destinationDirectory = this.backupService.getDestinationDirectory(hostname, backupNumber);

    return this.fileBrowserService.getFilesFromDirectory(Buffer.from(destinationDirectory)).pipe(
      map((file) => (file.name as unknown as Buffer).toString('latin1')),
      filter((file) => file.endsWith('.manifest')),
      map((file) => file.slice(0, -'.manifest'.length)),
    );
  }

  listFiles(hostname: string, backupNumber: number, sharePath: Buffer, path?: Buffer): AsyncIterableX<FileManifest> {
    const manifest = this.backupService.getManifest(hostname, backupNumber, sharePath);
    if (path?.length && path.compare(SEPARATOR, 0, SEPARATOR.length, 0, SEPARATOR.length) === 0) {
      path = path.subarray(SEPARATOR.length);
    }

    return this.manifestService.readManifestEntries(manifest).pipe(
      filter(
        (entry) =>
          !path || (path.length < entry.path.length && entry.path.compare(path, 0, path.length, 0, path.length) === 0),
      ),
      map((manifest) => ({
        ...manifest,
        path: (path && path.length && manifest.path.subarray(path.length)) || manifest.path,
      })),
      filter((manifest) => manifest.path.indexOf(SEPARATOR) === -1),
    );
  }
}
