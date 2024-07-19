import { Injectable } from '@nestjs/common';

import { PoolStatisticsService } from './pool-statistics.service.js';
import { PoolStatistics } from './statistics.interface.js';
import { HostsStatsUsage, StatsDiskUsage } from './statistics.interface.js';
import { ExecuteCommandService } from '../commands/execute-command.service.js';
import { HostsService } from '../backups/hosts.service.js';
import { BackupsService } from '../backups/backups.service.js';
import { concatMap, from, lastValueFrom, mergeMap, reduce } from 'rxjs';

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
    const host$ = from(this.hostsService.getHosts());
    const statUsage$ = host$.pipe(
      concatMap((hosts) => hosts),
      mergeMap(async (host) => {
        return {
          host,
          backups: await this.backupsService.getBackups(host),
          stats: await this.statsService.readHostStatistics(host),
        };
      }),
      reduce((acc, { host, backups, stats: hostStats }) => {
        if (backups.length > 0) {
          const lastBackup = backups[backups.length - 1];
          const stats = {
            backupCount: backups.length,
            lastBackupSize: lastBackup.fileSize,
            lastBackupTime: lastBackup.endDate || lastBackup.startDate || 0,
            lastBackupAge: new Date().getTime() - (lastBackup.endDate || lastBackup.startDate || 0),
            lastBackupDuration: (lastBackup.endDate || lastBackup.startDate) - lastBackup.startDate,
            lastBackupComplete: lastBackup.completed ? 1 : 0,
            ...hostStats,
          };
          acc[host] = stats;
        }
        return acc;
      }, {} as HostsStatsUsage),
    );
    return await lastValueFrom(statUsage$);
  }

  async getPoolStatsUsage(): Promise<PoolStatistics> {
    const poolStats = await this.statsService.readPoolStatistics();
    return poolStats;
  }
}
