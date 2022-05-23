import { StatsDiskUsage } from './stats.model';

export interface HistoricalDiskStatistics extends StatsDiskUsage {
  date: number;
}

export interface PoolStatistics {
  longestChain: number;
  nbChunk: number;
  nbRef: number;
  size: bigint;
  compressedSize: bigint;
}

export interface HistoricalPoolStatistics extends PoolStatistics {
  date: number;
}
