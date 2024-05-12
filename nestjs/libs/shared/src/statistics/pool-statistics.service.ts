import { Injectable } from '@nestjs/common';
import { join } from 'path';

import { HistoricalPoolStatistics, PoolStatistics } from './statistics.interface';
import { YamlService } from '../services';
import { BackupsService } from '../backups';
import { ApplicationConfigService } from '../config';

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

  async readStatistics(filename: string): Promise<PoolStatistics> {
    return this.yamlService.loadFile(filename, DEFAULT_STATISTICS);
  }

  async readHostStatistics(hostname: string): Promise<PoolStatistics> {
    const path = join(this.backupService.getHostPath(hostname), 'statistics.yml');
    return this.readStatistics(path);
  }

  async readBackupStatistics(hostname: string, backup: number): Promise<PoolStatistics> {
    const path = join(this.backupService.getBackupDestinationDirectory(hostname, backup), 'statistics.yml');
    return this.readStatistics(path);
  }

  async readPoolStatistics(): Promise<PoolStatistics> {
    const path = join(this.applicationConfig.poolPath, 'statistics.yml');
    return this.readStatistics(path);
  }

  async readHistoryStatistics(dirname: string): Promise<HistoricalPoolStatistics[]> {
    return this.yamlService.loadFile(join(dirname, 'history.yml'), [] as HistoricalPoolStatistics[]);
  }

  async readPoolHistoryStatistics(): Promise<HistoricalPoolStatistics[]> {
    return this.readHistoryStatistics(this.applicationConfig.poolPath);
  }

  async readBackupHistoryStatistics(hostname: string, backup: number): Promise<HistoricalPoolStatistics[]> {
    return this.readHistoryStatistics(this.backupService.getBackupDestinationDirectory(hostname, backup));
  }

  async readHostHistoryStatistics(hostname: string): Promise<HistoricalPoolStatistics[]> {
    return this.readHistoryStatistics(this.backupService.getHostPath(hostname));
  }
}
