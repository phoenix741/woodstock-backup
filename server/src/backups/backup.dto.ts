import { ObjectType } from '@nestjs/graphql';

@ObjectType()
export class DiskUsageStatisticsRecord {
  diskUsage!: number;
  uncompressed!: number;
}

@ObjectType()
export class DiskUsageStatistics {
  total?: DiskUsageStatisticsRecord;
  none?: DiskUsageStatisticsRecord;
  zlib?: DiskUsageStatisticsRecord;
  lzo?: DiskUsageStatisticsRecord;
  zstd?: DiskUsageStatisticsRecord;
}

@ObjectType()
export class Backup {
  number!: number;
  complete!: boolean;

  startDate!: number;
  endDate?: number;

  fileCount!: number;
  newFileCount!: number;
  existingFileCount!: number;

  fileSize!: number;
  existingFileSize!: number;
  newFileSize!: number;

  speed!: number;

  diskUsageStatistics?: DiskUsageStatistics;
}
