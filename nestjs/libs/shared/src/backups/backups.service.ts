import { Injectable } from '@nestjs/common';
import { CoreBackupsService, JsBackup } from '@woodstock/shared-rs';

@Injectable()
export class BackupsService {
  constructor(private backupsService: CoreBackupsService) {}

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
    return this.backupsService.getBackup(hostname, backupNumber);
  }

  getBackups(hostname: string): Promise<Array<JsBackup>> {
    return this.backupsService.getBackups(hostname);
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

  removeBackup(hostname: string, backupNumber: number): Promise<JsBackup> {
    return this.backupsService.removeBackup(hostname, backupNumber);
  }

  removeRefcntOfHost(hostname: string, backupNumber: number): Promise<void> {
    return this.backupsService.removeRefcntOfHost(hostname, backupNumber);
  }
}
