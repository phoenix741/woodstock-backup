import { Injectable, Logger } from '@nestjs/common';
import { copy } from 'fs-extra';
import { rm } from 'fs/promises';
import { join } from 'path';
import { ApplicationConfigService } from './application-config.service';
import { YamlService } from '../input-output';
import { Manifest } from '../manifest';
import { Backup } from '../shared';
import { mangle, unmangle } from '../utils';

@Injectable()
export class BackupsService {
  private logger = new Logger(BackupsService.name);

  constructor(private configService: ApplicationConfigService, private yamlService: YamlService) {}

  private getSharePathFile(hostname: string, backupNumber: number): string {
    return join(this.configService.hostPath, hostname, '' + backupNumber, 'shares.yml');
  }

  private getBackupFile(hostname: string): string {
    return join(this.configService.hostPath, hostname, 'backup.yml');
  }

  getLockFile(hostname: string): string {
    return join(this.configService.hostPath, hostname, 'LOCK');
  }

  getDestinationDirectory(hostname: string, backupNumber: number): string {
    return join(this.configService.hostPath, hostname, '' + backupNumber);
  }

  getManifest(hostname: string, backupNumber: number, share: Buffer): Manifest {
    return new Manifest(mangle(share), this.getDestinationDirectory(hostname, backupNumber));
  }

  /**
   * Return all the manifest of backup
   * @param hostname hostname of the backup
   * @param backupNumber number of the backup
   */
  async getManifests(hostname: string, backupNumber: number): Promise<Manifest[]> {
    return (await this.getBackupSharePaths(hostname, backupNumber)).map((share) =>
      this.getManifest(hostname, backupNumber, share),
    );
  }

  async addBackupSharePath(hostname: string, backupNumber: number, sharePath: Buffer): Promise<void> {
    const sharepathFilename = this.getSharePathFile(hostname, backupNumber);
    const sharepaths = new Set(await this.yamlService.loadFile<string[]>(sharepathFilename, []));

    sharepaths.add(mangle(sharePath));

    await this.yamlService.writeFile(sharepathFilename, [...sharepaths]);
  }

  async getBackupSharePaths(hostname: string, backupNumber: number): Promise<Buffer[]> {
    const sharepathFilename = this.getSharePathFile(hostname, backupNumber);
    return await (await this.yamlService.loadFile<string[]>(sharepathFilename, [])).map((path) => unmangle(path));
  }

  getHostDirectory(hostname: string): string {
    return join(this.configService.hostPath, hostname);
  }

  getLogDirectory(hostname: string): string {
    return join(this.configService.hostPath, hostname, 'logs');
  }

  getLogFile(hostname: string, backupNumber?: number, type?: string): string {
    return join(
      this.configService.hostPath,
      hostname,
      'logs',
      `backup.${backupNumber !== undefined ? backupNumber + '.' : ''}${type ? type + '.' : ''}log`,
    );
  }

  async getBackups(hostname: string): Promise<Backup[]> {
    const backups = await this.yamlService.loadFile<Backup[]>(this.getBackupFile(hostname), []);

    return backups.map((backup) => {
      if ((backup.startDate as any) instanceof Date) {
        backup.startDate = (backup.startDate as any as Date).getTime();
      }
      if ((backup.endDate as any) instanceof Date) {
        backup.endDate = (backup.endDate as any as Date).getTime();
      }
      return backup;
    });
  }

  async getBackup(hostname: string, number: number): Promise<Backup | undefined> {
    const backups = await this.getBackups(hostname);
    return backups.find((b) => b.number === number);
  }

  async getLastBackup(hostname: string): Promise<Backup | undefined> {
    const backups = await this.getBackups(hostname);
    return backups.length ? backups[backups.length - 1] : undefined;
  }

  async addOrReplaceBackup(hostname: string, backup: Backup): Promise<void> {
    const backups = await this.getBackups(hostname);

    const foundIndex = backups.findIndex((element) => element.number === backup.number);

    if (foundIndex >= 0) {
      backups.splice(foundIndex, 1, backup);
    } else {
      backups.push(backup);
    }

    await this.yamlService.writeFile(this.getBackupFile(hostname), backups);
  }

  async removeBackup(hostname: string, number: number): Promise<void> {
    const backups = await this.getBackups(hostname);

    const foundIndex = backups.findIndex((element) => element.number === number);

    if (foundIndex >= 0) {
      backups.splice(foundIndex, 1);
    }

    await this.yamlService.writeFile(this.getBackupFile(hostname), backups);

    await rm(this.getDestinationDirectory(hostname, number), { recursive: true });
  }

  async cloneBackup(hostname: string, number: number | undefined, destinationNumber: number): Promise<void> {
    if (number !== undefined) {
      try {
        const source = this.getDestinationDirectory(hostname, number);
        const destination = this.getDestinationDirectory(hostname, destinationNumber);

        // Copy all files from source directory to destination directory expect history.yml
        await copy(source, destination);
      } catch (err) {
        this.logger.error(
          `Can't start the backup ${destinationNumber} for ${hostname} from previous backup ${number}: ${err.message}`,
          err.stack,
        );
      }
    }
  }
}
