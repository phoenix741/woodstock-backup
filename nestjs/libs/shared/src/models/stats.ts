import { StatsDiskUsage } from './stats.model.js';

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
