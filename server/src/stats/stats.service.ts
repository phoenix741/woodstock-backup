import { Injectable } from '@nestjs/common';

import { DiskUsageStatistics } from '../backups/backup.dto';
import { ExecuteCommandService } from '../operation/execute-command.service';
import { CommandParameters } from '../server/tools.model';
import { BackupQuota } from './stats.model';

@Injectable()
export class StatsService {
  constructor(private executeCommandService: ExecuteCommandService) {}

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
    const volumes = await this.listSubvolume(params);
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

      const volumeId = parseInt(id.replace('0/', ''));

      acc[volumeId] = { refr: parseInt(refr), excl: parseInt(excl) };
      return acc;
    }, {} as Record<number, { refr: number; excl: number }>);
  }

  private async listSubvolume(params: CommandParameters) {
    const { stdout } = await this.executeCommandService.executeTool('btrfsListSubvolume', params);
    const [, , ...lines] = stdout.toString().split(/[\n\r]/);

    return lines.reduce((acc, line) => {
      const [id, , , path] = line.split(/\s+/).filter(n => !!n);
      if (!id) {
        return acc;
      }

      const pathArray = path.split('/').slice(-2);
      const host = pathArray[0];
      const number = parseInt(pathArray[1]);

      acc[parseInt(id)] = { host, number };
      return acc;
    }, {} as Record<number, { host: string; number: number }>);
  }
}
