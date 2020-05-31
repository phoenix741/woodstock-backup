import { ObjectType } from '@nestjs/graphql';

@ObjectType()
export class SpaceStatistics {
  timestamp!: number;
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
export class TimestampBackupQuota {
  timestamp!: number;
  volumes!: BackupQuota[];
}

@ObjectType()
export class Statistics {
  spaces!: SpaceStatistics[];
  quotas!: TimestampBackupQuota[];
}
