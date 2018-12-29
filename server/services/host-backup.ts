import * as path from 'path'
import * as fs from 'fs'
import * as yaml from 'js-yaml'

import { NG_MASS_BACKUP_HOST_PATH } from '../config/index'

export interface Backup {
  number: number
  isBackupFull: boolean

  startDate: Date
  endDate: Date

  fileCount: number
  newFileCount: number
  existingFileCount: number

  fileSize: number
  existingFileSize: number
  newFileSize: number

  speed: number
}

export class HostBackup {
  private _backups: Array<Backup>

  constructor (private _host: string) {}

  get directory () {
    return path.join(NG_MASS_BACKUP_HOST_PATH, this._host)
  }

  get backupFile () {
    return path.join(this.directory, 'backup.yml')
  }

  getDestinationDirectory (backupNumber: number) {
    return path.join(NG_MASS_BACKUP_HOST_PATH, this._host, '' + backupNumber)
  }

  get backups (): Promise<Array<Backup>> {
    if (! this._backups) {
      return this.loadBackups().then(() => this._backups)
    }
    return Promise.resolve(this._backups)
  }

  get lastBackup (): Promise<Backup | null> {
    return this.backups.then(backups => backups.length ? backups[backups.length - 1] : null)
  }

  private async loadBackups () {
    try {
      const backupsFromFile = await fs.promises.readFile(this.backupFile, 'utf8')
      this._backups = yaml.safeLoad(backupsFromFile) || []
    } catch (err) {
      this._backups = []
    }
  }

  private async saveBackups () {
    const backupsFromStr = yaml.safeDump(this._backups)
    await fs.promises.writeFile(this.backupFile, backupsFromStr, 'utf-8')
  }

  async addBackup (backup: Backup): Promise<void> {
    await this.loadBackups()

    const foundIndex = this._backups.findIndex(element => element.number === backup.number)
    if (foundIndex >= 0) {
      this._backups.splice(foundIndex, 1, backup)
    } else {
      this._backups.push(backup)
    }

    await this.saveBackups()
  }
}
