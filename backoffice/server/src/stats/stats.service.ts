import { Injectable } from '@nestjs/common';
import { BackupsService } from '../backups/backups.service';
import { ApplicationConfigService } from '../config/application-config.service';
import { HostsService } from '../hosts/hosts.service';
import { CommandParameters } from '../server/tools.model';
import { ExecuteCommandService } from '../utils/execute-command.service';
import { YamlService } from '../utils/yaml.service';
import { BackupQuota, CompressionStatistics, DiskUsageStats } from './stats.model';

const DEFAULT_STATISTICS: DiskUsageStats = {
  quotas: [],
};

@Injectable()
export class StatsService {
  private stats: DiskUsageStats | null = null;
  private statsLoaded = false;

  constructor(
    private executeCommandService: ExecuteCommandService,
    private configService: ApplicationConfigService,
    private yamlService: YamlService,
    private hostsService: HostsService,
    private backupsService: BackupsService,
  ) {}

  async getCompressionStatistics(): Promise<CompressionStatistics[]> {
    const hosts = await this.hostsService.getHosts();
    const backups = (await Promise.all(hosts.map(async (hostname) => await this.backupsService.getBackups(hostname))))
      .flat()
      .sort((b1, b2) => b1.startDate - b2.startDate);

    const total = { timestamp: 0, diskUsage: 0, uncompressed: 0 };
    return backups.reduce((acc, v) => {
      total.timestamp = v.startDate;
      // total.diskUsage += v.diskUsageStatistics?.total?.diskUsage || 0;
      // total.uncompressed += v.diskUsageStatistics?.total?.uncompressed || 0;
      acc.push(Object.assign({}, total));
      return acc;
    }, [] as CompressionStatistics[]);
  }

  async getStatistics(): Promise<DiskUsageStats> {
    if (this.statsLoaded === false) {
      this.stats = await this.yamlService.loadFile(this.configService.statisticsPath, DEFAULT_STATISTICS);
      this.statsLoaded = true;
    }

    return this.stats || DEFAULT_STATISTICS;
  }

  async refreshStatistics(): Promise<void> {
    const params: CommandParameters = {};
    const space = await this.getSpace(params);
    const volumes = await this.getBackupQuota(params);

    const statistics = await this.yamlService.loadFile(this.configService.statisticsPath, DEFAULT_STATISTICS);
    statistics.quotas.push({ timestamp: new Date().getTime(), volumes, space });

    await this.yamlService.writeFile(this.configService.statisticsPath, statistics);
    this.statsLoaded = false;
  }

  async getSpace(params: CommandParameters): Promise<{ fstype: string; size: number; used: number; free: number }> {
    const { stdout } = await this.executeCommandService.executeTool('statsSpaceUsage', params);
    const [, line] = stdout.toString().split(/[\n\r]/);
    const [, fstype, nbBlock1K, nbUsedBlock, nbFreeBlock] = line.split(/\s+/).filter((n) => !!n);
    return {
      fstype,
      size: parseInt(nbBlock1K) * 1024,
      used: parseInt(nbUsedBlock) * 1024,
      free: parseInt(nbFreeBlock) * 1024,
    };
  }

  async getBackupQuota(params: CommandParameters): Promise<BackupQuota[]> {
    const result: BackupQuota[] = [];

    return result;
  }
}
