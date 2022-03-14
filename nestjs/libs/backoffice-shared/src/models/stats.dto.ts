import { ObjectType } from '@nestjs/graphql';

/**
 * Represent the pool size. The file can be in the pool size or in a subdirectory
 * of the pool size.
 */
@ObjectType()
export class PoolSize {
  fileCount: bigint;
  poolSize: bigint;
  compressedPoolSize: bigint;
}

@ObjectType()
export class CompressionStatistics {
  timestamp!: number;
  diskUsage!: number;
  uncompressed!: number;
}

@ObjectType()
export class SpaceStatistics {
  size!: number;
  used!: number;
  free!: number;
}

@ObjectType()
export class BackupQuota {
  host!: string;
  number!: number;
  refr!: number;
  excl!: number;
}

@ObjectType()
export class HostQuota {
  host!: string;
  excl!: number;
  refr!: number;
  total!: number;
}

@ObjectType()
export class TotalQuota {
  refr!: number;
  excl!: number;
  total!: number;
}

@ObjectType()
export class TimestampBackupQuota {
  timestamp!: number;
  volumes!: BackupQuota[];
  space!: SpaceStatistics;
}

@ObjectType()
export class DiskUsageStats {
  quotas!: TimestampBackupQuota[];
}
