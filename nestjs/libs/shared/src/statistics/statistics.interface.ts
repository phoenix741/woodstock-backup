export interface StatsDiskUsage {
  fstype: string;
  size: bigint;
  used: bigint;
  free: bigint;
}

export interface HistoricalDiskStatistics extends StatsDiskUsage {
  date: number;
}

export class PoolStatistics {
  longestChain = 0;
  nbChunk = 0;
  nbRef = 0;
  size = 0n;
  compressedSize = 0n;
  unusedSize = 0n;
}

export interface HistoricalPoolStatistics extends PoolStatistics {
  date: number;
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
