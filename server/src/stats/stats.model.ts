import { ObjectType } from '@nestjs/graphql';

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
