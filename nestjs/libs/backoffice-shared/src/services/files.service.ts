import { Injectable } from '@nestjs/common';
import { FileBrowserService } from '@woodstock/shared';
import { AsyncIterableX } from 'ix/asynciterable';
import { filter, map } from 'ix/asynciterable/operators';
import { BackupsService } from './backups.service';

@Injectable()
export class FilesService {
  constructor(private backupService: BackupsService, private fileBrowserService: FileBrowserService) {}

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
}
