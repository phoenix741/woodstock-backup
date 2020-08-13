import { Injectable } from '@nestjs/common';
import { JobId } from 'bull';
import { join } from 'path';

import { ApplicationConfigService } from '../config/application-config.service';
import { LockService } from '../utils/lock.service';
import { YamlService } from '../utils/yaml.service';
import { Backup } from './backup.dto';

@Injectable()
export class BackupsService {
  constructor(
    private configService: ApplicationConfigService,
    private yamlService: YamlService,
    private lockService: LockService,
  ) {}

  private getBackupFile(hostname: string) {
    return join(this.configService.hostPath, hostname, 'backup.yml');
  }

  private getLockFile(hostname: string) {
    return join(this.configService.hostPath, hostname, 'LOCK');
  }

  getDestinationDirectory(hostname: string, backupNumber: number) {
    return join(this.configService.hostPath, hostname, '' + backupNumber);
  }

  getLogDirectory(hostname: string) {
    return join(this.configService.hostPath, hostname, 'logs');
  }

  getLogFile(hostname: string, backupNumber?: number, type?: string) {
    return join(
      this.configService.hostPath,
      hostname,
      'logs',
      `backup.${backupNumber !== undefined ? backupNumber + '.' : ''}${type ? type + '.' : ''}log`,
    );
  }

  async getBackups(hostname: string): Promise<Backup[]> {
    const backups = await this.yamlService.loadFile<Backup[]>(this.getBackupFile(hostname), []);

    return backups.map(backup => {
      if ((backup.startDate as any) instanceof Date) {
        backup.startDate = ((backup.startDate as any) as Date).getTime();
      }
      if ((backup.endDate as any) instanceof Date) {
        backup.endDate = ((backup.endDate as any) as Date).getTime();
      }
      return backup;
    });
  }

  async getBackup(hostname: string, number: number): Promise<Backup | undefined> {
    const backups = await this.getBackups(hostname);
    return backups.find(b => b.number === number);
  }

  async getLastBackup(hostname: string): Promise<Backup | undefined> {
    const backups = await this.getBackups(hostname);
    return backups.length ? backups[backups.length - 1] : undefined;
  }

  async addBackup(hostname: string, backup: Backup): Promise<void> {
    const backups = await this.getBackups(hostname);

    const foundIndex = backups.findIndex(element => element.number === backup.number);

    if (foundIndex >= 0) {
      backups.splice(foundIndex, 1, backup);
    } else {
      backups.push(backup);
    }

    await this.yamlService.writeFile(this.getBackupFile(hostname), backups);
  }

  async removeBackup(hostname: string, number: number) {
    const backups = await this.getBackups(hostname);

    const foundIndex = backups.findIndex(element => element.number === number);

    if (foundIndex >= 0) {
      backups.splice(foundIndex, 1);
    }

    await this.yamlService.writeFile(this.getBackupFile(hostname), backups);
  }

  async lock(hostname: string, jobId: JobId, force = false): Promise<JobId | null> {
    return await this.lockService.lock(this.getLockFile(hostname), jobId, force);
  }

  async isLocked(hostname: string): Promise<JobId | null> {
    return await this.lockService.isLocked(this.getLockFile(hostname));
  }

  async unlock(hostname: string, jobId: JobId, force = false): Promise<JobId | null> {
    return await this.lockService.unlock(this.getLockFile(hostname), jobId, force);
  }
}
