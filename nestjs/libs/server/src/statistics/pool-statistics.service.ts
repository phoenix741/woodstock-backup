import { Injectable } from '@nestjs/common';
import { join } from 'path';
import { ApplicationConfigService, YamlService } from '@woodstock/core';
import { HistoricalPoolStatistics, PoolStatistics } from './statistics.interface';
import { BackupsService } from '../backups/backups.service';

export const DEFAULT_STATISTICS: PoolStatistics = {
  longestChain: 0,
  nbRef: 0,
  nbChunk: 0,
  compressedSize: 0n,
  size: 0n,
  unusedSize: 0n,
};

@Injectable()
export class PoolStatisticsService {
  constructor(
    private readonly yamlService: YamlService,
    private readonly backupService: BackupsService,
    private readonly applicationConfig: ApplicationConfigService,
  ) {}

  async writeStatistics(statistics: PoolStatistics, dirname: string, fakeDate?: number): Promise<void> {
    await this.yamlService.writeFile(join(dirname, 'statistics.yml'), statistics);
    await this.appendHistoryStatistics(dirname, statistics, fakeDate);
  }

  async readStatistics(filename: string): Promise<PoolStatistics> {
    return this.yamlService.loadFile(filename, DEFAULT_STATISTICS);
  }

  async readHostStatistics(hostname: string): Promise<PoolStatistics> {
    const path = join(this.backupService.getHostDirectory(hostname), 'statistics.yml');
    return this.readStatistics(path);
  }

  async readBackupStatistics(hostname: string, backup: number): Promise<PoolStatistics> {
    const path = join(this.backupService.getDestinationDirectory(hostname, backup), 'statistics.yml');
    return this.readStatistics(path);
  }

  async readPoolStatistics(): Promise<PoolStatistics> {
    const path = join(this.applicationConfig.poolPath, 'statistics.yml');
    return this.readStatistics(path);
  }

  async appendHistoryStatistics(dirname: string, statistics: PoolStatistics, fakeDate?: number): Promise<void> {
    const path = join(dirname, 'history.yml');
    const histories = await this.yamlService.loadFile(path, [] as HistoricalPoolStatistics[]);
    histories.push({ ...statistics, date: fakeDate ?? new Date().getTime() });

    await this.yamlService.writeFile(path, histories);
  }

  async readHistoryStatistics(dirname: string): Promise<HistoricalPoolStatistics[]> {
    return this.yamlService.loadFile(join(dirname, 'history.yml'), [] as HistoricalPoolStatistics[]);
  }

  async readPoolHistoryStatistics(): Promise<HistoricalPoolStatistics[]> {
    return this.readHistoryStatistics(this.applicationConfig.poolPath);
  }

  async readBackupHistoryStatistics(hostname: string, backup: number): Promise<HistoricalPoolStatistics[]> {
    return this.readHistoryStatistics(this.backupService.getDestinationDirectory(hostname, backup));
  }

  async readHostHistoryStatistics(hostname: string): Promise<HistoricalPoolStatistics[]> {
    return this.readHistoryStatistics(this.backupService.getHostDirectory(hostname));
  }
}
