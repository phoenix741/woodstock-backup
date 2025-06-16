import { Injectable } from '@nestjs/common';
import { statfs } from 'fs/promises';
import { concatMap, from, lastValueFrom, mergeMap, reduce } from 'rxjs';
import { BackupsService } from '../backups/backups.service.js';
import { HostsService } from '../backups/hosts.service.js';
import { ApplicationConfigService } from '../config/application-config.service.js';
import { PoolStatisticsService } from './pool-statistics.service.js';
import { HostsStatsUsage, PoolStatistics, StatsDiskUsage } from './statistics.interface.js';
import { JsBackupStatus } from '@woodstock/shared-rs';

const FS_MAPPING = new Map<number, string>([
  [0xadf5, 'ADFS'],
  [0xadff, 'AFFS'],
  [0x5346414f, 'AFS'],
  [0x09041934, 'ANON_INODE_FS'],
  [0x0187, 'AUTOFS'],
  [0x62646576, 'BDEVFS'],
  [0x42465331, 'BEFS'],
  [0x1badface, 'BFS'],
  [0x42494e4d, 'BINFMTFS'],
  [0xcafe4a11, 'BPF_FS'],
  [0x9123683e, 'BTRFS'],
  [0x73727279, 'BTRFS_TEST'],
  [0x27e0eb, 'CGROUP'],
  [0x63677270, 'CGROUP2'],
  [0xff534d42, 'CIFS_NUMBER'],
  [0x73757245, 'CODA'],
  [0x012ff7b7, 'COH'],
  [0x28cd3d45, 'CRAMFS'],
  [0x64626720, 'DEBUGFS'],
  [0x1373, 'DEVFS'],
  [0x1cd1, 'DEVPTS'],
  [0xf15f, 'ECRYPTFS'],
  [0xde5e81e4, 'EFIVARFS'],
  [0x00414a53, 'EFS'],
  [0x137d, 'EXT'],
  [0xef51, 'EXT2_OLD'],
  [0xef53, 'EXT2'],
  [0xef53, 'EXT3'],
  [0xef53, 'EXT4'],
  [0xf2f52010, 'F2FS'],
  [0x65735546, 'FUSE'],
  [0xbad1dea, 'FUTEXFS'],
  [0x4244, 'HFS'],
  [0x00c0ffee, 'HOSTFS'],
  [0xf995e849, 'HPFS'],
  [0x958458f6, 'HUGETLBFS'],
  [0x9660, 'ISOFS'],
  [0x72b6, 'JFFS2'],
  [0x3153464a, 'JFS'],
  [0x137f, 'MINIX'],
  [0x138f, 'MINIX2'],
  [0x2468, 'MINIX2'],
  [0x2478, 'MINIX22'],
  [0x4d5a, 'MINIX3'],
  [0x19800202, 'MQUEUE'],
  [0x4d44, 'MSDOS'],
  [0x11307854, 'MTD_INODE_FS'],
  [0x564c, 'NCP'],
  [0x6969, 'NFS'],
  [0x3434, 'NILFS'],
  [0x6e736673, 'NSFS'],
  [0x5346544e, 'NTFS_SB'],
  [0x7461636f, 'OCFS2'],
  [0x9fa1, 'OPENPROM'],
  [0x794c7630, 'OVERLAYFS'],
  [0x50495045, 'PIPEFS'],
  [0x9fa0, 'PROC'],
  [0x6165676c, 'PSTOREFS'],
  [0x002f, 'QNX4'],
  [0x68191122, 'QNX6'],
  [0x858458f6, 'RAMFS'],
  [0x52654973, 'REISERFS'],
  [0x7275, 'ROMFS'],
  [0x73636673, 'SECURITYFS'],
  [0xf97cff8c, 'SELINUX'],
  [0x43415d53, 'SMACK'],
  [0x517b, 'SMB'],
  [0xfe534d42, 'SMB2_NUMBER'],
  [0x534f434b, 'SOCKFS'],
  [0x73717368, 'SQUASHFS'],
  [0x62656572, 'SYSFS'],
  [0x012ff7b6, 'SYSV2'],
  [0x012ff7b5, 'SYSV4'],
  [0x01021994, 'TMPFS'],
  [0x74726163, 'TRACEFS'],
  [0x15013346, 'UDF'],
  [0x00011954, 'UFS'],
  [0x9fa2, 'USBDEVICE'],
  [0x01021997, 'V9FS'],
  [0xa501fcf5, 'VXFS'],
  [0xabba1974, 'XENFS'],
  [0x012ff7b4, 'XENIX'],
  [0x58465342, 'XFS'],
  [0x012fd16d, '_XIAFS'],
]);

@Injectable()
export class StatsInstantService {
  constructor(
    private config: ApplicationConfigService,
    private hostsService: HostsService,
    private backupsService: BackupsService,
    private statsService: PoolStatisticsService,
  ) {}

  async getSpace(): Promise<StatsDiskUsage> {
    const stats = await statfs(this.config.poolPath, { bigint: true });
    return {
      fstype: FS_MAPPING.get(Number(stats.type)) ?? 'UNKNOWN',
      size: stats.blocks * stats.bsize,
      used: stats.bavail * stats.bsize,
      free: (stats.blocks - stats.bavail) * stats.bsize,
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
            lastBackupComplete: [JsBackupStatus.Completed].includes(lastBackup.status) ? 1 : 0,
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
