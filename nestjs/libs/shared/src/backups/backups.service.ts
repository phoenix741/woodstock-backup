import { CACHE_MANAGER } from '@nestjs/cache-manager';
import { Inject, Injectable } from '@nestjs/common';
import { CoreBackupsService, JsBackup } from '@woodstock/shared-rs';
import { Cache } from 'cache-manager';

@Injectable()
export class BackupsService {
  constructor(
    @Inject(CACHE_MANAGER) private cacheManager: Cache,
    private backupsService: CoreBackupsService,
  ) {}

  getBackupDestinationDirectory(hostname: string, backupNumber: number): string {
    return this.backupsService.getBackupDestinationDirectory(hostname, backupNumber);
  }

  getLogDirectory(hostname: string, backupNumber: number): string {
    return this.backupsService.getLogDirectory(hostname, backupNumber);
  }

  getHostPath(hostname: string): string {
    return this.backupsService.getHostPath(hostname);
  }

  getBackup(hostname: string, backupNumber: number): Promise<JsBackup | null> {
    return this.cacheManager.wrap(`backup-${hostname}-${backupNumber}`, () =>
      this.backupsService.getBackup(hostname, backupNumber),
    );
  }

  getBackups(hostname: string): Promise<Array<JsBackup>> {
    return this.cacheManager.wrap(`backups-${hostname}`, () => this.backupsService.getBackups(hostname));
  }

  getLastBackup(hostname: string): Promise<JsBackup | null> {
    return this.backupsService.getLastBackup(hostname);
  }

  getPreviousBackup(hostname: string, backupNumber: number): Promise<JsBackup | null> {
    return this.backupsService.getPreviousBackup(hostname, backupNumber);
  }

  getBackupSharePaths(hostname: string, backupNumber: number): Promise<Array<string>> {
    return this.backupsService.getBackupSharePaths(hostname, backupNumber);
  }

  async getTimeSinceLastBackup(hostname: string): Promise<number | undefined> {
    const lastBackup = await this.backupsService.getLastBackup(hostname);
    if (!lastBackup) {
      return undefined;
    }

    return new Date().getTime() / 1000 - (lastBackup?.startDate ?? 0);
  }

  async removeBackup(hostname: string, backupNumber: number): Promise<JsBackup> {
    const backup = this.backupsService.removeBackup(hostname, backupNumber);
    await this.invalidateBackup(hostname, backupNumber);
    return backup;
  }

  removeRefcntOfHost(hostname: string, backupNumber: number): Promise<void> {
    return this.backupsService.removeRefcntOfHost(hostname, backupNumber);
  }

  async invalidateBackup(hostname: string, backupNumber?: number): Promise<void> {
    if (backupNumber) {
      await this.cacheManager.del(`backup-${hostname}-${backupNumber}`);
    }
    await this.cacheManager.del(`backups-${hostname}`);
  }
}
