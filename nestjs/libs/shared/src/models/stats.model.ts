import { PoolStatistics } from './stats';

export interface StatsDiskUsage {
  fstype: string;
  size: bigint;
  used: bigint;
  free: bigint;
}

export interface HostStatsUsage extends PoolStatistics {
  backupCount: number;
  lastBackupSize: bigint;
  lastBackupTime: number;
  lastBackupAge: number;
  lastBackupDuration: number;
  lastBackupComplete: number;
}

export interface HostsStatsUsage {
  [host: string]: HostStatsUsage;
}
