import { Injectable, Logger } from '@nestjs/common';
import {
  ApplicationConfigService,
  BackupsService,
  ManifestService,
  RefCntService,
  ReferenceCount,
} from '@woodstock/shared';

@Injectable()
export class RemoveService {
  #logger = new Logger(RemoveService.name);

  constructor(
    private applicationConfig: ApplicationConfigService,
    private backupsService: BackupsService,
    private manifestService: ManifestService,
    private refcntService: RefCntService,
  ) {}

  async remove(hostname: string, n: number) {
    this.#logger.log(`Removing backup ${n} of ${hostname}`);
    try {
      const manifests = await this.backupsService.getManifests(hostname, n);
      for (const manifest of manifests) {
        await this.manifestService.deleteManifest(manifest);
      }

      const refcnt = new ReferenceCount(
        this.backupsService.getHostDirectory(hostname),
        this.backupsService.getDestinationDirectory(hostname, n),
        this.applicationConfig.poolPath,
      );

      // We start by cleaning the refcnt file of the host
      await this.refcntService.removeBackupRefcntTo(refcnt.hostPath, refcnt.backupPath);

      // We start by create the unused file for the pool
      await this.refcntService.removeBackupRefcntTo(refcnt.poolPath, refcnt.backupPath, refcnt.unusedPoolPath);

      // Remove backup
      await this.backupsService.removeBackup(hostname, n);
    } catch (err) {
      this.#logger.error(`Error while removing backup ${n} of ${hostname}`, err);
    } finally {
      this.#logger.log(`[END] Removing backup ${n} of ${hostname} done`);
    }
  }
}
