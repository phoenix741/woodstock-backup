import { Injectable } from '@nestjs/common';
import { join } from 'path';
import { ApplicationConfigService } from '../config';
import { YamlService } from '../input-output';
import { HistoricalDiskStatistics, StatsDiskUsage } from './statistics.interface';

@Injectable()
export class DiskStatisticsService {
  constructor(
    private readonly yamlService: YamlService,
    private readonly applicationConfig: ApplicationConfigService,
  ) {}

  async appendHistoryStatistics(statistics: StatsDiskUsage): Promise<void> {
    const path = join(this.applicationConfig.poolPath, 'disk_history.yml');
    const histories = await this.yamlService.loadFile(path, [] as HistoricalDiskStatistics[]);
    histories.push({ ...statistics, date: new Date().getTime() });

    await this.yamlService.writeFile(path, histories);
  }

  async readHistoryStatistics(): Promise<HistoricalDiskStatistics[]> {
    return this.yamlService.loadFile(
      join(this.applicationConfig.poolPath, 'disk_history.yml'),
      [] as HistoricalDiskStatistics[],
    );
  }
}
