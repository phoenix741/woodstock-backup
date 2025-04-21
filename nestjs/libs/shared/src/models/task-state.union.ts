import { createUnionType } from '@nestjs/graphql';
import { JobBackupData, JobCleanupData, JobFsckData, JobRemoveData, JobRestoreData } from '../backuping/backuping.dto';
import { BackupTaskState } from './backup-tasks.dto';
import { CleanerTaskState } from './cleaner-tasks.dto';
import { FsckTaskState } from './fsck-tasks.dto';
import { RemoveTaskState } from './remove-tasks.dto';
import { RestoreTaskState } from './restore-tasks.dto';

export const BackupQueueDataUnion = createUnionType({
  name: 'BackupQueueData',
  types: () => [JobBackupData, JobRestoreData, JobRemoveData, JobCleanupData, JobFsckData] as const,
});

export const TaskStateUnion = createUnionType({
  name: 'TaskState',
  types: () => [BackupTaskState, RestoreTaskState, RemoveTaskState, CleanerTaskState, FsckTaskState] as const,
});

export type BackupProgressData =
  | BackupTaskState
  | RestoreTaskState
  | RemoveTaskState
  | CleanerTaskState
  | FsckTaskState;
