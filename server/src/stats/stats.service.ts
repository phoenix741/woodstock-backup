import { Injectable } from '@nestjs/common';

import { DiskUsageStatistics } from '../backups/backup.dto';
import { BackupsService } from '../backups/backups.service';
import { ApplicationConfigService } from '../config/application-config.service';
import { HostsService } from '../hosts/hosts.service';
import { ExecuteCommandService } from '../operation/execute-command.service';
import { CommandParameters } from '../server/tools.model';
import { BtrfsService } from '../storage/btrfs/btrfs.service';
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
    private btrfsService: BtrfsService,
  ) {}

  async getCompressionStatistics() {
    const hosts = await this.hostsService.getHosts();
    const backups = (await Promise.all(hosts.map(async hostname => await this.backupsService.getBackups(hostname))))
      .flat()
      .sort((b1, b2) => b1.startDate - b2.startDate);

    const total = { timestamp: 0, diskUsage: 0, uncompressed: 0 };
    return backups.reduce((acc, v) => {
      total.timestamp = v.startDate;
      total.diskUsage += v.diskUsageStatistics?.total?.diskUsage || 0;
      total.uncompressed += v.diskUsageStatistics?.total?.uncompressed || 0;
      acc.push(Object.assign({}, total));
      return acc;
    }, [] as CompressionStatistics[]);
  }

  async getStatistics() {
    if (this.statsLoaded === false) {
      this.stats = await this.yamlService.loadFile(this.configService.statisticsPath, DEFAULT_STATISTICS);
      this.statsLoaded = true;
    }

    return this.stats || DEFAULT_STATISTICS;
  }

  async refreshStatistics() {
    const params: CommandParameters = {};
    const space = await this.getSpace(params);
    const volumes = await this.getBackupQuota(params);

    const statistics = await this.yamlService.loadFile(this.configService.statisticsPath, DEFAULT_STATISTICS);
    statistics.quotas.push({ timestamp: new Date().getTime(), volumes, space });

    await this.yamlService.writeFile(this.configService.statisticsPath, statistics);
    this.statsLoaded = false;
  }

  async getSpace(params: CommandParameters) {
    const { stdout } = await this.executeCommandService.executeTool('statsSpaceUsage', params);
    const [, line] = stdout.toString().split(/[\n\r]/);
    const [, fstype, nbBlock1K, nbUsedBlock, nbFreeBlock] = line.split(/\s+/).filter(n => !!n);
    return {
      fstype,
      size: parseInt(nbBlock1K) * 1024,
      used: parseInt(nbUsedBlock) * 1024,
      free: parseInt(nbFreeBlock) * 1024,
    };
  }

  async getBackupQuota(params: CommandParameters) {
    const volumes = { ...(await this.listSubvolume(params)), ...(await this.listHost(params)) };
    const quotas = await this.listQuota(params);

    const result: BackupQuota[] = [];

    for (const volumeId in volumes) {
      const volume = volumes[volumeId];
      const quota = quotas[volumeId];
      result.push({
        ...volume,
        ...quota,
      });
    }

    return result;
  }

  async calculateCompressionSize(params: CommandParameters) {
    const { stdout } = await this.executeCommandService.executeTool('btrfsGetCompressionSize', params);
    const [, , ...lines] = stdout.toString().split(/[\n\r]/);

    return lines.reduce((acc, line) => {
      const [type, , diskUsage, uncompressed] = line.split(/\s+/).filter(n => !!n);
      if (!type) {
        return acc;
      }

      acc[type.toLowerCase() as keyof DiskUsageStatistics] = {
        diskUsage: parseInt(diskUsage),
        uncompressed: parseInt(uncompressed),
      };
      return acc;
    }, {} as DiskUsageStatistics);
  }

  private async listQuota(params: CommandParameters) {
    const { stdout } = await this.executeCommandService.executeTool('statsDiskUsage', params);
    const [, , ...lines] = stdout.toString().split(/[\n\r]/);

    return lines.reduce((acc, line) => {
      const [id, refr, excl] = line.split(/\s+/).filter(n => !!n);
      if (!id) {
        return acc;
      }

      acc[id] = { refr: parseInt(refr), excl: parseInt(excl) };
      return acc;
    }, {} as Record<string, { refr: number; excl: number }>);
  }

  private async listSubvolume(params: CommandParameters) {
    const lines = await this.btrfsService.listSubvolume(params);
    return lines.reduce((acc, line) => {
      const { id, host, number } = line;

      acc[`0/${id}`] = { host, number };
      return acc;
    }, {} as Record<string, { host: string; number: number }>);
  }

  private async listHost(params: CommandParameters) {
    const hosts = await this.hostsService.getHosts();
    return await hosts.reduce(async (accP, host) => {
      const acc = await accP;
      const qGroupId = await this.btrfsService.getHostGroupId({ ...params, hostname: host });
      if (qGroupId >= 0) {
        acc[`1/${qGroupId}`] = { host, number: -1 };
      }
      return acc;
    }, Promise.resolve({} as Record<string, { host: string; number: number }>));
  }
}
