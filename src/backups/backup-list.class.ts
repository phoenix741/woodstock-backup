import { Logger } from '@nestjs/common';
import * as fs from 'fs';
import * as yaml from 'js-yaml';
import { join } from 'path';

import { Backup } from './backup.dto';

export class BackupList {
  private logger = new Logger(BackupList.name);

  private _backups: Array<Backup> = [];

  constructor(private _hostPath: string, private _host: string) {}

  get directory() {
    return join(this._hostPath, this._host);
  }

  get backupFile() {
    return join(this.directory, 'backup.yml');
  }

  get lockFile() {
    return join(this.directory, 'LOCK');
  }

  getDestinationDirectory(backupNumber: number) {
    return join(this._hostPath, this._host, '' + backupNumber);
  }

  async getBackups(): Promise<Backup[]> {
    if (!this._backups.length) {
      await this.loadBackups();
    }
    return this._backups;
  }

  async getLastBackup(): Promise<Backup | null> {
    const backups = await this.getBackups();
    return backups.length ? backups[backups.length - 1] : null;
  }

  async lock(jobId: string, force = false): Promise<string | null> {
    try {
      await fs.promises.writeFile(this.lockFile, jobId, { encoding: 'utf-8', flag: force ? 'w' : 'wx' });

      return null;
    } catch (err) {
      if (err.code === 'EEXIST') {
        const previousLock = await fs.promises.readFile(this.lockFile, 'utf-8');
        if (previousLock === jobId) {
          return await this.lock(jobId, true);
        }
        return previousLock;
      }
      throw err;
    }
  }

  async unlock(jobId: string, force = false): Promise<string | null> {
    try {
      const currentLock = await fs.promises.readFile(this.lockFile, 'utf-8');
      if (currentLock !== jobId && !force) {
        return currentLock;
      }

      await fs.promises.unlink(this.lockFile);
      return null;
    } catch (err) {
      if (err.code === 'ENOENT') {
        // Not locked
        return null;
      }
      throw err;
    }
  }

  async addBackup(backup: Backup): Promise<void> {
    await this.loadBackups();

    const foundIndex = this._backups.findIndex(element => element.number === backup.number);

    if (foundIndex >= 0) {
      this._backups.splice(foundIndex, 1, backup);
    } else {
      this._backups.push(backup);
    }

    await this.saveBackups();
  }

  private async loadBackups() {
    this.logger.log(`Load backup for the host ${this._host}.`);
    try {
      const backupsFromFile = await fs.promises.readFile(this.backupFile, 'utf8');
      this._backups = yaml.safeLoad(backupsFromFile) || [];
      this.logger.log(`Found ${this._backups.length} for the host ${this._host}`);
    } catch (err) {
      this._backups = [];
      this.logger.warn(`Can't load backups for the host ${this._host}: ${err.message}`);
    }
  }

  private async saveBackups() {
    this.logger.log(`Save backup for the host ${this._host}.`);
    const backupsFromStr = yaml.safeDump(this._backups);
    await fs.promises.writeFile(this.backupFile, backupsFromStr, 'utf-8');
  }
}
