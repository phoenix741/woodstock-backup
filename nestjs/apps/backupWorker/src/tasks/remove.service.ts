import { Injectable, Logger } from '@nestjs/common';
import { ApplicationConfigService, BackupsService, RefCntService, ReferenceCount } from '@woodstock/shared';

@Injectable()
export class RemoveService {
  #logger = new Logger(RemoveService.name);

  constructor(
    private applicationConfig: ApplicationConfigService,
    private backupsService: BackupsService,
    private refcntService: RefCntService,
  ) {}

  async remove(hostname: string, backupNumber: number) {
    this.#logger.log(`Removing backup ${backupNumber} of ${hostname}`);
    try {
      const refcnt = new ReferenceCount(
        this.backupsService.getHostDirectory(hostname),
        this.backupsService.getDestinationDirectory(hostname, backupNumber),
        this.applicationConfig.poolPath,
      );

      // We start by cleaning the refcnt file of the host
      await this.refcntService.removeBackupRefcntTo(refcnt.hostPath, refcnt.backupPath);

      // Remove backup
      await this.backupsService.removeBackup(hostname, backupNumber);
    } catch (err) {
      this.#logger.error(`Error while removing backup ${backupNumber} of ${hostname}`, err);
      throw err;
    } finally {
      this.#logger.log(`[END] Removing backup ${backupNumber} of ${hostname} done`);
    }
  }
}
