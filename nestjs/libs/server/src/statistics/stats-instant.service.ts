import { Injectable } from '@nestjs/common';
import { from, reduce } from 'ix/asynciterable';
import { concatMap } from 'ix/asynciterable/operators';
import { ExecuteCommandService } from '@woodstock/core';
import { PoolStatisticsService } from './pool-statistics.service.js';
import { PoolStatistics } from './statistics.interface';
import { HostsStatsUsage, StatsDiskUsage } from './statistics.interface.js';
import { HostsService } from '../backups/hosts.service.js';
import { BackupsService } from '../backups/backups.service.js';

@Injectable()
export class StatsInstantService {
  constructor(
    private executeCommandService: ExecuteCommandService,
    private hostsService: HostsService,
    private backupsService: BackupsService,
    private statsService: PoolStatisticsService,
  ) {}

  async getSpace(): Promise<StatsDiskUsage> {
    const { stdout } = await this.executeCommandService.executeTool('statsSpaceUsage', {});
    const [, line] = stdout.toString().split(/[\n\r]/);
    const [, fstype, nbBlock1K, nbUsedBlock, nbFreeBlock] = line.split(/\s+/).filter((n) => !!n);
    return {
      fstype,
      size: BigInt(nbBlock1K) * 1024n,
      used: BigInt(nbUsedBlock) * 1024n,
      free: BigInt(nbFreeBlock) * 1024n,
    };
  }

  async getHostsStatsUsage(): Promise<HostsStatsUsage> {
    return await reduce(from(this.hostsService.getHosts()).pipe(concatMap((hosts) => from(hosts))), {
      callback: async (acc, host) => {
        const backups = await this.backupsService.getBackups(host);
        const hostStats = await this.statsService.readHostStatistics(host);

        if (backups.length > 0) {
          const lastBackup = backups[backups.length - 1];
          const stats = {
            backupCount: backups.length,
            lastBackupSize: lastBackup.fileSize,
            lastBackupTime: lastBackup.endDate || lastBackup.startDate || 0,
            lastBackupAge: new Date().getTime() - (lastBackup.endDate || lastBackup.startDate || 0),
            lastBackupDuration: (lastBackup.endDate || lastBackup.startDate) - lastBackup.startDate,
            lastBackupComplete: lastBackup.complete ? 1 : 0,
            ...hostStats,
          };
          acc[host] = stats;
        }
        return acc;
      },
      seed: {} as HostsStatsUsage,
    });
  }

  async getPoolStatsUsage(): Promise<PoolStatistics> {
    const poolStats = await this.statsService.readPoolStatistics();
    return poolStats;
  }
}
