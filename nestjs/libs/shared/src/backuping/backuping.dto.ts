import { ObjectType } from '@nestjs/graphql';
import { HostConfiguration } from '../models';

@ObjectType()
export class JobBackupData {
  host!: string;
  config?: HostConfiguration;
  previousNumber?: number;
  number?: number;
  ip?: string;
  startDate?: number;

  force?: boolean;
}

@ObjectType()
export class JobRestoreDataSelection {
  share: string;
  selection: string[];
}

@ObjectType()
export class JobRestoreData {
  host!: string;
  config?: HostConfiguration;
  number?: number;
  ip?: string;
  startDate?: number;

  // Restoration
  destinationDirectory: string;
  files: JobRestoreDataSelection[];
}

@ObjectType()
export class JobRemoveData {
  host!: string;
  config?: HostConfiguration;
  number?: number;
  startDate?: number;
}

@ObjectType()
export class JobCleanupData {
  target?: string;
}

@ObjectType()
export class JobFsckData {
  dryRun: boolean;
  verifyChunks: boolean;
}

export type BackupQueueData = JobBackupData | JobRestoreData | JobRemoveData | JobCleanupData | JobFsckData;
