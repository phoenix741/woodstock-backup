/* eslint-disable */
import type { TypedDocumentNode as DocumentNode } from '@graphql-typed-document-node/core';
export type Maybe<T> = T | null;
export type InputMaybe<T> = Maybe<T>;
export type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: string;
  String: string;
  Boolean: boolean;
  Int: number;
  Float: number;
  /** The `BigInt` scalar type represents non-fractional signed whole numeric values. BigInt can represent values between -(2^63) + 1 and 2^63 - 1. */
  BigInt: bigint;
};

export type Backup = {
  __typename?: 'Backup';
  complete: Scalars['Boolean'];
  compressedFileSize: Scalars['BigInt'];
  endDate?: Maybe<Scalars['Float']>;
  existingCompressedFileSize: Scalars['BigInt'];
  existingFileCount: Scalars['Float'];
  existingFileSize: Scalars['BigInt'];
  fileCount: Scalars['Float'];
  fileSize: Scalars['BigInt'];
  files: Array<FileDescription>;
  newCompressedFileSize: Scalars['BigInt'];
  newFileCount: Scalars['Float'];
  newFileSize: Scalars['BigInt'];
  number: Scalars['Float'];
  shares: Array<FileDescription>;
  speed: Scalars['Float'];
  startDate: Scalars['Float'];
};


export type BackupFilesArgs = {
  path: Scalars['String'];
  sharePath: Scalars['String'];
};

export type BackupOperation = {
  __typename?: 'BackupOperation';
  excludes?: Maybe<Array<Scalars['String']>>;
  includes?: Maybe<Array<Scalars['String']>>;
  shares: Array<BackupTaskShare>;
  timeout?: Maybe<Scalars['Float']>;
};

export type BackupTask = {
  __typename?: 'BackupTask';
  description?: Maybe<Scalars['String']>;
  groupName?: Maybe<Scalars['String']>;
  host?: Maybe<Scalars['String']>;
  ip?: Maybe<Scalars['String']>;
  number?: Maybe<Scalars['Float']>;
  progression?: Maybe<JobProgression>;
  startDate?: Maybe<Scalars['Float']>;
  state?: Maybe<QueueTaskState>;
  subtasks: Array<SubTaskOrGroupTasks>;
};

/**
 * Part of config file.
 *
 * Store information about a share
 */
export type BackupTaskShare = {
  __typename?: 'BackupTaskShare';
  excludes?: Maybe<Array<Scalars['String']>>;
  includes?: Maybe<Array<Scalars['String']>>;
  name: Scalars['String'];
};

export type BigIntTimeSerie = {
  __typename?: 'BigIntTimeSerie';
  time: Scalars['Float'];
  value: Scalars['BigInt'];
};

/**
 * Part of config file
 *
 * Store information about a DHCP Address
 */
export type DhcpAddress = {
  __typename?: 'DhcpAddress';
  address: Scalars['String'];
  end: Scalars['Float'];
  start: Scalars['Float'];
};

export type DiskUsage = {
  __typename?: 'DiskUsage';
  free?: Maybe<Scalars['BigInt']>;
  freeLastMonth?: Maybe<Scalars['BigInt']>;
  freeRange?: Maybe<Array<BigIntTimeSerie>>;
  total?: Maybe<Scalars['BigInt']>;
  totalLastMonth?: Maybe<Scalars['BigInt']>;
  totalRange?: Maybe<Array<BigIntTimeSerie>>;
  used?: Maybe<Scalars['BigInt']>;
  usedLastMonth?: Maybe<Scalars['BigInt']>;
  usedRange?: Maybe<Array<BigIntTimeSerie>>;
};

export enum EnumFileType {
  BlockDevice = 'BLOCK_DEVICE',
  CharacterDevice = 'CHARACTER_DEVICE',
  Directory = 'DIRECTORY',
  Fifo = 'FIFO',
  RegularFile = 'REGULAR_FILE',
  Share = 'SHARE',
  Socket = 'SOCKET',
  SymbolicLink = 'SYMBOLIC_LINK',
  Unknown = 'UNKNOWN'
}

export type ExecuteCommandOperation = {
  __typename?: 'ExecuteCommandOperation';
  command: Scalars['String'];
};

export type FileAcl = {
  __typename?: 'FileAcl';
  group?: Maybe<Scalars['String']>;
  mask?: Maybe<Scalars['Float']>;
  other?: Maybe<Scalars['Float']>;
  user?: Maybe<Scalars['String']>;
};

export type FileDescription = {
  __typename?: 'FileDescription';
  acl: Array<FileAcl>;
  path: Scalars['String'];
  stats?: Maybe<FileStat>;
  symlink?: Maybe<Scalars['String']>;
  type: EnumFileType;
  xattr: Scalars['String'];
};

export type FileStat = {
  __typename?: 'FileStat';
  compressedSize?: Maybe<Scalars['String']>;
  created?: Maybe<Scalars['String']>;
  dev?: Maybe<Scalars['String']>;
  groupId?: Maybe<Scalars['String']>;
  ino?: Maybe<Scalars['String']>;
  lastModified?: Maybe<Scalars['String']>;
  lastRead?: Maybe<Scalars['String']>;
  mode?: Maybe<Scalars['String']>;
  nlink?: Maybe<Scalars['String']>;
  ownerId?: Maybe<Scalars['String']>;
  rdev?: Maybe<Scalars['String']>;
  size?: Maybe<Scalars['String']>;
};

export type Host = {
  __typename?: 'Host';
  backups: Array<Backup>;
  configuration: HostConfiguration;
  lastBackup?: Maybe<Backup>;
  lastBackupState?: Maybe<Scalars['String']>;
  name: Scalars['String'];
};

export type HostConfigOperation = {
  __typename?: 'HostConfigOperation';
  operation?: Maybe<BackupOperation>;
  postCommands?: Maybe<Array<ExecuteCommandOperation>>;
  preCommands?: Maybe<Array<ExecuteCommandOperation>>;
};

/**
 * Config file for one Host
 *
 * Contains all information that can be used to backup a host.
 */
export type HostConfiguration = {
  __typename?: 'HostConfiguration';
  addresses?: Maybe<Array<Scalars['String']>>;
  dhcp?: Maybe<Array<DhcpAddress>>;
  isLocal?: Maybe<Scalars['Boolean']>;
  operations?: Maybe<HostConfigOperation>;
  password: Scalars['String'];
  schedule?: Maybe<Schedule>;
};

export type HostStatistics = {
  __typename?: 'HostStatistics';
  compressedSize?: Maybe<Scalars['BigInt']>;
  compressedSizeLastMonth?: Maybe<Scalars['BigInt']>;
  compressedSizeRange?: Maybe<Array<BigIntTimeSerie>>;
  host?: Maybe<Scalars['String']>;
  longestChain?: Maybe<Scalars['Int']>;
  longestChainLastMonth?: Maybe<Scalars['Int']>;
  longestChainRange?: Maybe<Array<NumberTimeSerie>>;
  nbChunk?: Maybe<Scalars['Int']>;
  nbChunkLastMonth?: Maybe<Scalars['Int']>;
  nbChunkRange?: Maybe<Array<NumberTimeSerie>>;
  nbRef?: Maybe<Scalars['Int']>;
  nbRefLastMonth?: Maybe<Scalars['Int']>;
  nbRefRange?: Maybe<Array<NumberTimeSerie>>;
  size?: Maybe<Scalars['BigInt']>;
  sizeLastMonth?: Maybe<Scalars['BigInt']>;
  sizeRange?: Maybe<Array<BigIntTimeSerie>>;
};

export type Job = {
  __typename?: 'Job';
  attemptsMade: Scalars['Int'];
  data: BackupTask;
  failedReason?: Maybe<Scalars['String']>;
  id?: Maybe<Scalars['String']>;
  name: Scalars['String'];
  queueName: Scalars['String'];
  state: Scalars['String'];
};

export type JobGroupTasks = {
  __typename?: 'JobGroupTasks';
  description?: Maybe<Scalars['String']>;
  groupName?: Maybe<Scalars['String']>;
  progression?: Maybe<JobProgression>;
  state?: Maybe<QueueTaskState>;
  subtasks: Array<SubTaskOrGroupTasks>;
};

export type JobProgression = {
  __typename?: 'JobProgression';
  compressedFileSize?: Maybe<Scalars['BigInt']>;
  errorCount?: Maybe<Scalars['Int']>;
  fileCount?: Maybe<Scalars['Int']>;
  fileSize?: Maybe<Scalars['BigInt']>;
  newCompressedFileSize?: Maybe<Scalars['BigInt']>;
  newFileCount?: Maybe<Scalars['Int']>;
  newFileSize?: Maybe<Scalars['BigInt']>;
  percent?: Maybe<Scalars['Float']>;
  progressCurrent?: Maybe<Scalars['BigInt']>;
  progressMax?: Maybe<Scalars['BigInt']>;
  speed?: Maybe<Scalars['Float']>;
};

export type JobResponse = {
  __typename?: 'JobResponse';
  id: Scalars['String'];
};

export type JobSubTask = {
  __typename?: 'JobSubTask';
  description?: Maybe<Scalars['String']>;
  progression?: Maybe<JobProgression>;
  state?: Maybe<QueueTaskState>;
  taskName: Scalars['String'];
};

export type Mutation = {
  __typename?: 'Mutation';
  checkAndFixPool: JobResponse;
  cleanupPool: JobResponse;
  createBackup: JobResponse;
  removeBackup: JobResponse;
  verifyChecksum: JobResponse;
};


export type MutationCheckAndFixPoolArgs = {
  fix: Scalars['Boolean'];
};


export type MutationCreateBackupArgs = {
  hostname: Scalars['String'];
};


export type MutationRemoveBackupArgs = {
  hostname: Scalars['String'];
  number: Scalars['Int'];
};

export type NumberTimeSerie = {
  __typename?: 'NumberTimeSerie';
  time: Scalars['Float'];
  value: Scalars['Int'];
};

export type PoolUsage = {
  __typename?: 'PoolUsage';
  compressedSize?: Maybe<Scalars['BigInt']>;
  compressedSizeLastMonth?: Maybe<Scalars['BigInt']>;
  compressedSizeRange?: Maybe<Array<BigIntTimeSerie>>;
  longestChain?: Maybe<Scalars['Int']>;
  longestChainLastMonth?: Maybe<Scalars['Int']>;
  longestChainRange?: Maybe<Array<NumberTimeSerie>>;
  nbChunk?: Maybe<Scalars['Int']>;
  nbChunkLastMonth?: Maybe<Scalars['Int']>;
  nbChunkRange?: Maybe<Array<NumberTimeSerie>>;
  nbRef?: Maybe<Scalars['Int']>;
  nbRefLastMonth?: Maybe<Scalars['Int']>;
  nbRefRange?: Maybe<Array<NumberTimeSerie>>;
  size?: Maybe<Scalars['BigInt']>;
  sizeLastMonth?: Maybe<Scalars['BigInt']>;
  sizeRange?: Maybe<Array<BigIntTimeSerie>>;
  unusedSize?: Maybe<Scalars['BigInt']>;
  unusedSizeLastMonth?: Maybe<Scalars['BigInt']>;
  unusedSizeRange?: Maybe<Array<BigIntTimeSerie>>;
};

export type Query = {
  __typename?: 'Query';
  backup: Backup;
  backups: Array<Backup>;
  host: Host;
  hosts: Array<Host>;
  queue: Array<Job>;
  queueStats: QueueStats;
  statistics: Statistics;
};


export type QueryBackupArgs = {
  hostname: Scalars['String'];
  number: Scalars['Int'];
};


export type QueryBackupsArgs = {
  hostname: Scalars['String'];
};


export type QueryHostArgs = {
  hostname: Scalars['String'];
};


export type QueryQueueArgs = {
  input: QueueListInput;
};

export type QueueListInput = {
  operationName?: InputMaybe<Scalars['String']>;
  queueName?: InputMaybe<Scalars['String']>;
  states?: Array<Scalars['String']>;
};

export type QueueStats = {
  __typename?: 'QueueStats';
  active: Scalars['Int'];
  completed: Scalars['Int'];
  delayed: Scalars['Int'];
  failed: Scalars['Int'];
  lastExecution?: Maybe<Scalars['Float']>;
  nextWakeup?: Maybe<Scalars['Float']>;
  waiting: Scalars['Int'];
  waitingChildren: Scalars['Int'];
};

export enum QueueTaskState {
  Aborted = 'ABORTED',
  Failed = 'FAILED',
  Running = 'RUNNING',
  Success = 'SUCCESS',
  Waiting = 'WAITING'
}

export type Schedule = {
  __typename?: 'Schedule';
  activated?: Maybe<Scalars['Boolean']>;
  backupPeriod?: Maybe<Scalars['Float']>;
  backupToKeep?: Maybe<ScheduledBackupToKeep>;
};

export type ScheduledBackupToKeep = {
  __typename?: 'ScheduledBackupToKeep';
  daily?: Maybe<Scalars['Float']>;
  hourly?: Maybe<Scalars['Float']>;
  monthly?: Maybe<Scalars['Float']>;
  weekly?: Maybe<Scalars['Float']>;
  yearly?: Maybe<Scalars['Float']>;
};

export type Statistics = {
  __typename?: 'Statistics';
  diskUsage?: Maybe<DiskUsage>;
  hosts?: Maybe<Array<HostStatistics>>;
  poolUsage?: Maybe<PoolUsage>;
};

export type SubTaskOrGroupTasks = JobGroupTasks | JobSubTask;

export type Subscription = {
  __typename?: 'Subscription';
  jobFailed: Job;
  jobRemoved: Job;
  jobUpdated: Job;
  jobWaiting: Scalars['Int'];
};

export type HostsQueryVariables = Exact<{ [key: string]: never; }>;


export type HostsQuery = { __typename?: 'Query', hosts: Array<{ __typename?: 'Host', name: string, lastBackupState?: string | null, lastBackup?: { __typename?: 'Backup', number: number, startDate: number, fileSize: bigint, complete: boolean } | null, configuration: { __typename?: 'HostConfiguration', schedule?: { __typename?: 'Schedule', activated?: boolean | null } | null } }> };

export type BackupsQueryVariables = Exact<{
  hostname: Scalars['String'];
}>;


export type BackupsQuery = { __typename?: 'Query', backups: Array<{ __typename?: 'Backup', number: number, complete: boolean, startDate: number, endDate?: number | null, fileCount: number, newFileCount: number, existingFileCount: number, fileSize: bigint, newFileSize: bigint, existingFileSize: bigint, speed: number }> };

export type BackupsBrowseQueryVariables = Exact<{
  hostname: Scalars['String'];
  number: Scalars['Int'];
  sharePath: Scalars['String'];
  path: Scalars['String'];
}>;


export type BackupsBrowseQuery = { __typename?: 'Query', backup: { __typename?: 'Backup', files: Array<(
      { __typename?: 'FileDescription' }
      & { ' $fragmentRefs'?: { 'FragmentFileDescriptionFragment': FragmentFileDescriptionFragment } }
    )> } };

export type CreateBackupMutationVariables = Exact<{
  hostname: Scalars['String'];
}>;


export type CreateBackupMutation = { __typename?: 'Mutation', createBackup: { __typename?: 'JobResponse', id: string } };

export type RemoveBackupMutationVariables = Exact<{
  hostname: Scalars['String'];
  number: Scalars['Int'];
}>;


export type RemoveBackupMutation = { __typename?: 'Mutation', removeBackup: { __typename?: 'JobResponse', id: string } };

export type FragmentFileDescriptionFragment = { __typename?: 'FileDescription', path: string, type: EnumFileType, symlink?: string | null, stats?: { __typename?: 'FileStat', ownerId?: string | null, groupId?: string | null, mode?: string | null, size?: string | null, lastModified?: string | null } | null } & { ' $fragmentName'?: 'FragmentFileDescriptionFragment' };

export type SharesBrowseQueryVariables = Exact<{
  hostname: Scalars['String'];
  number: Scalars['Int'];
}>;


export type SharesBrowseQuery = { __typename?: 'Query', backup: { __typename?: 'Backup', shares: Array<(
      { __typename?: 'FileDescription' }
      & { ' $fragmentRefs'?: { 'FragmentFileDescriptionFragment': FragmentFileDescriptionFragment } }
    )> } };

export type CleanupPoolMutationVariables = Exact<{ [key: string]: never; }>;


export type CleanupPoolMutation = { __typename?: 'Mutation', cleanupPool: { __typename?: 'JobResponse', id: string } };

export type FsckPoolMutationVariables = Exact<{
  fix: Scalars['Boolean'];
}>;


export type FsckPoolMutation = { __typename?: 'Mutation', checkAndFixPool: { __typename?: 'JobResponse', id: string } };

export type VerifyChecksumMutationVariables = Exact<{ [key: string]: never; }>;


export type VerifyChecksumMutation = { __typename?: 'Mutation', verifyChecksum: { __typename?: 'JobResponse', id: string } };

export type DiskUsageStatisticsQueryVariables = Exact<{ [key: string]: never; }>;


export type DiskUsageStatisticsQuery = { __typename?: 'Query', statistics: { __typename?: 'Statistics', hosts?: Array<{ __typename?: 'HostStatistics', host?: string | null, size?: bigint | null, compressedSize?: bigint | null }> | null } };

export type PoolStatisticsQueryVariables = Exact<{ [key: string]: never; }>;


export type PoolStatisticsQuery = { __typename?: 'Query', statistics: { __typename?: 'Statistics', diskUsage?: { __typename?: 'DiskUsage', used?: bigint | null, usedLastMonth?: bigint | null, free?: bigint | null, total?: bigint | null } | null, poolUsage?: { __typename?: 'PoolUsage', nbChunk?: number | null, nbChunkLastMonth?: number | null, nbRef?: number | null, nbRefLastMonth?: number | null, size?: bigint | null, compressedSize?: bigint | null, compressedSizeLastMonth?: bigint | null, unusedSize?: bigint | null, nbChunkRange?: Array<{ __typename?: 'NumberTimeSerie', time: number, value: number }> | null, compressedSizeRange?: Array<{ __typename?: 'BigIntTimeSerie', time: number, value: bigint }> | null } | null } };

export type QueueStatisticsQueryVariables = Exact<{ [key: string]: never; }>;


export type QueueStatisticsQuery = { __typename?: 'Query', queueStats: { __typename?: 'QueueStats', active: number, waiting: number, failed: number, delayed: number, completed: number } };

export type ProgressTaskFragment = { __typename?: 'JobProgression', progressCurrent?: bigint | null, progressMax?: bigint | null, fileSize?: bigint | null, newFileSize?: bigint | null, compressedFileSize?: bigint | null, newCompressedFileSize?: bigint | null, fileCount?: number | null, newFileCount?: number | null, errorCount?: number | null } & { ' $fragmentName'?: 'ProgressTaskFragment' };

type TaskDescription_JobGroupTasks_Fragment = { __typename: 'JobGroupTasks' } & { ' $fragmentName'?: 'TaskDescription_JobGroupTasks_Fragment' };

type TaskDescription_JobSubTask_Fragment = { __typename: 'JobSubTask', taskName: string, description?: string | null, state?: QueueTaskState | null } & { ' $fragmentName'?: 'TaskDescription_JobSubTask_Fragment' };

export type TaskDescriptionFragment = TaskDescription_JobGroupTasks_Fragment | TaskDescription_JobSubTask_Fragment;

type BackupTask_JobGroupTasks_Fragment = { __typename: 'JobGroupTasks', groupName?: string | null, description?: string | null, state?: QueueTaskState | null, progression?: (
    { __typename?: 'JobProgression' }
    & { ' $fragmentRefs'?: { 'ProgressTaskFragment': ProgressTaskFragment } }
  ) | null, taskDescription: Array<(
    { __typename?: 'JobGroupTasks' }
    & { ' $fragmentRefs'?: { 'TaskDescription_JobGroupTasks_Fragment': TaskDescription_JobGroupTasks_Fragment } }
  ) | (
    { __typename?: 'JobSubTask' }
    & { ' $fragmentRefs'?: { 'TaskDescription_JobSubTask_Fragment': TaskDescription_JobSubTask_Fragment } }
  )> } & { ' $fragmentName'?: 'BackupTask_JobGroupTasks_Fragment' };

type BackupTask_JobSubTask_Fragment = { __typename: 'JobSubTask', taskName: string, description?: string | null, state?: QueueTaskState | null, progression?: (
    { __typename?: 'JobProgression' }
    & { ' $fragmentRefs'?: { 'ProgressTaskFragment': ProgressTaskFragment } }
  ) | null } & { ' $fragmentName'?: 'BackupTask_JobSubTask_Fragment' };

export type BackupTaskFragment = BackupTask_JobGroupTasks_Fragment | BackupTask_JobSubTask_Fragment;

export type JobProgressionFragment = { __typename?: 'JobProgression', newFileCount?: number | null, fileCount?: number | null, progressCurrent?: bigint | null, progressMax?: bigint | null, speed?: number | null } & { ' $fragmentName'?: 'JobProgressionFragment' };

export type JobFragment = { __typename?: 'Job', id?: string | null, queueName: string, name: string, failedReason?: string | null, state: string, data: { __typename?: 'BackupTask', host?: string | null, number?: number | null, startDate?: number | null, groupName?: string | null, description?: string | null, ip?: string | null, state?: QueueTaskState | null, progression?: (
      { __typename?: 'JobProgression' }
      & { ' $fragmentRefs'?: { 'JobProgressionFragment': JobProgressionFragment } }
    ) | null, subtasks: Array<(
      { __typename?: 'JobGroupTasks' }
      & { ' $fragmentRefs'?: { 'BackupTask_JobGroupTasks_Fragment': BackupTask_JobGroupTasks_Fragment } }
    ) | (
      { __typename?: 'JobSubTask' }
      & { ' $fragmentRefs'?: { 'BackupTask_JobSubTask_Fragment': BackupTask_JobSubTask_Fragment } }
    )> } } & { ' $fragmentName'?: 'JobFragment' };

export type TasksQueryVariables = Exact<{
  input: QueueListInput;
}>;


export type TasksQuery = { __typename?: 'Query', queue: Array<(
    { __typename?: 'Job' }
    & { ' $fragmentRefs'?: { 'JobFragment': JobFragment } }
  )> };

export type QueueTasksJobUpdatedSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type QueueTasksJobUpdatedSubscription = { __typename?: 'Subscription', jobUpdated: (
    { __typename?: 'Job' }
    & { ' $fragmentRefs'?: { 'JobFragment': JobFragment } }
  ) };

export const FragmentFileDescriptionFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"FragmentFileDescription"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"FileDescription"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"path"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"stats"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"ownerId"}},{"kind":"Field","name":{"kind":"Name","value":"groupId"}},{"kind":"Field","name":{"kind":"Name","value":"mode"}},{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"lastModified"}}]}},{"kind":"Field","name":{"kind":"Name","value":"symlink"}}]}}]} as unknown as DocumentNode<FragmentFileDescriptionFragment, unknown>;
export const JobProgressionFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobProgression"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobProgression"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}}]}}]} as unknown as DocumentNode<JobProgressionFragment, unknown>;
export const ProgressTaskFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ProgressTask"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobProgression"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}}]}}]} as unknown as DocumentNode<ProgressTaskFragment, unknown>;
export const TaskDescriptionFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"TaskDescription"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"SubTaskOrGroupTasks"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobSubTask"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"taskName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"state"}}]}}]}}]} as unknown as DocumentNode<TaskDescriptionFragment, unknown>;
export const BackupTaskFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BackupTask"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"SubTaskOrGroupTasks"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobGroupTasks"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"groupName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProgressTask"}}]}},{"kind":"Field","alias":{"kind":"Name","value":"taskDescription"},"name":{"kind":"Name","value":"subtasks"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"TaskDescription"}}]}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobSubTask"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"taskName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProgressTask"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ProgressTask"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobProgression"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"TaskDescription"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"SubTaskOrGroupTasks"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobSubTask"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"taskName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"state"}}]}}]}}]} as unknown as DocumentNode<BackupTaskFragment, unknown>;
export const JobFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Job"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Job"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"queueName"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"failedReason"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"groupName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"ip"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobProgression"}}]}},{"kind":"Field","name":{"kind":"Name","value":"subtasks"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BackupTask"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ProgressTask"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobProgression"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"TaskDescription"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"SubTaskOrGroupTasks"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobSubTask"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"taskName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"state"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobProgression"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobProgression"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BackupTask"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"SubTaskOrGroupTasks"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobGroupTasks"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"groupName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProgressTask"}}]}},{"kind":"Field","alias":{"kind":"Name","value":"taskDescription"},"name":{"kind":"Name","value":"subtasks"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"TaskDescription"}}]}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobSubTask"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"taskName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProgressTask"}}]}}]}}]}}]} as unknown as DocumentNode<JobFragment, unknown>;
export const HostsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"Hosts"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"hosts"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"lastBackup"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"complete"}}]}},{"kind":"Field","name":{"kind":"Name","value":"lastBackupState"}},{"kind":"Field","name":{"kind":"Name","value":"configuration"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"schedule"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"activated"}}]}}]}}]}}]}}]} as unknown as DocumentNode<HostsQuery, HostsQueryVariables>;
export const BackupsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"Backups"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"backups"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"hostname"},"value":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"complete"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"endDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"existingFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"existingFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}}]}}]}}]} as unknown as DocumentNode<BackupsQuery, BackupsQueryVariables>;
export const BackupsBrowseDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"BackupsBrowse"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"number"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"sharePath"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"path"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"backup"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"hostname"},"value":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}}},{"kind":"Argument","name":{"kind":"Name","value":"number"},"value":{"kind":"Variable","name":{"kind":"Name","value":"number"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"files"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"sharePath"},"value":{"kind":"Variable","name":{"kind":"Name","value":"sharePath"}}},{"kind":"Argument","name":{"kind":"Name","value":"path"},"value":{"kind":"Variable","name":{"kind":"Name","value":"path"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"FragmentFileDescription"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"FragmentFileDescription"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"FileDescription"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"path"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"stats"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"ownerId"}},{"kind":"Field","name":{"kind":"Name","value":"groupId"}},{"kind":"Field","name":{"kind":"Name","value":"mode"}},{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"lastModified"}}]}},{"kind":"Field","name":{"kind":"Name","value":"symlink"}}]}}]} as unknown as DocumentNode<BackupsBrowseQuery, BackupsBrowseQueryVariables>;
export const CreateBackupDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"createBackup"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"createBackup"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"hostname"},"value":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]} as unknown as DocumentNode<CreateBackupMutation, CreateBackupMutationVariables>;
export const RemoveBackupDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"removeBackup"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"number"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"removeBackup"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"hostname"},"value":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}}},{"kind":"Argument","name":{"kind":"Name","value":"number"},"value":{"kind":"Variable","name":{"kind":"Name","value":"number"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]} as unknown as DocumentNode<RemoveBackupMutation, RemoveBackupMutationVariables>;
export const SharesBrowseDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"SharesBrowse"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"number"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"backup"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"hostname"},"value":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}}},{"kind":"Argument","name":{"kind":"Name","value":"number"},"value":{"kind":"Variable","name":{"kind":"Name","value":"number"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"shares"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"FragmentFileDescription"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"FragmentFileDescription"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"FileDescription"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"path"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"stats"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"ownerId"}},{"kind":"Field","name":{"kind":"Name","value":"groupId"}},{"kind":"Field","name":{"kind":"Name","value":"mode"}},{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"lastModified"}}]}},{"kind":"Field","name":{"kind":"Name","value":"symlink"}}]}}]} as unknown as DocumentNode<SharesBrowseQuery, SharesBrowseQueryVariables>;
export const CleanupPoolDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"cleanupPool"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"cleanupPool"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]} as unknown as DocumentNode<CleanupPoolMutation, CleanupPoolMutationVariables>;
export const FsckPoolDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"fsckPool"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"fix"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Boolean"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"checkAndFixPool"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"fix"},"value":{"kind":"Variable","name":{"kind":"Name","value":"fix"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]} as unknown as DocumentNode<FsckPoolMutation, FsckPoolMutationVariables>;
export const VerifyChecksumDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"verifyChecksum"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"verifyChecksum"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]} as unknown as DocumentNode<VerifyChecksumMutation, VerifyChecksumMutationVariables>;
export const DiskUsageStatisticsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"DiskUsageStatistics"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"statistics"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"hosts"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"compressedSize"}}]}}]}}]}}]} as unknown as DocumentNode<DiskUsageStatisticsQuery, DiskUsageStatisticsQueryVariables>;
export const PoolStatisticsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"PoolStatistics"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"statistics"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"diskUsage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"used"}},{"kind":"Field","name":{"kind":"Name","value":"usedLastMonth"}},{"kind":"Field","name":{"kind":"Name","value":"free"}},{"kind":"Field","name":{"kind":"Name","value":"total"}}]}},{"kind":"Field","name":{"kind":"Name","value":"poolUsage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"nbChunk"}},{"kind":"Field","name":{"kind":"Name","value":"nbChunkLastMonth"}},{"kind":"Field","name":{"kind":"Name","value":"nbChunkRange"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"time"}},{"kind":"Field","name":{"kind":"Name","value":"value"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nbRef"}},{"kind":"Field","name":{"kind":"Name","value":"nbRefLastMonth"}},{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"compressedSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedSizeLastMonth"}},{"kind":"Field","name":{"kind":"Name","value":"compressedSizeRange"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"time"}},{"kind":"Field","name":{"kind":"Name","value":"value"}}]}},{"kind":"Field","name":{"kind":"Name","value":"unusedSize"}}]}}]}}]}}]} as unknown as DocumentNode<PoolStatisticsQuery, PoolStatisticsQueryVariables>;
export const QueueStatisticsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"QueueStatistics"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"queueStats"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"active"}},{"kind":"Field","name":{"kind":"Name","value":"waiting"}},{"kind":"Field","name":{"kind":"Name","value":"failed"}},{"kind":"Field","name":{"kind":"Name","value":"delayed"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}}]}}]}}]} as unknown as DocumentNode<QueueStatisticsQuery, QueueStatisticsQueryVariables>;
export const TasksDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"Tasks"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"QueueListInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"queue"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Job"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobProgression"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobProgression"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ProgressTask"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobProgression"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"TaskDescription"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"SubTaskOrGroupTasks"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobSubTask"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"taskName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"state"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BackupTask"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"SubTaskOrGroupTasks"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobGroupTasks"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"groupName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProgressTask"}}]}},{"kind":"Field","alias":{"kind":"Name","value":"taskDescription"},"name":{"kind":"Name","value":"subtasks"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"TaskDescription"}}]}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobSubTask"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"taskName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProgressTask"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Job"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Job"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"queueName"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"failedReason"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"groupName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"ip"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobProgression"}}]}},{"kind":"Field","name":{"kind":"Name","value":"subtasks"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BackupTask"}}]}}]}}]}}]} as unknown as DocumentNode<TasksQuery, TasksQueryVariables>;
export const QueueTasksJobUpdatedDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"subscription","name":{"kind":"Name","value":"QueueTasksJobUpdated"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"jobUpdated"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Job"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobProgression"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobProgression"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ProgressTask"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobProgression"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"TaskDescription"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"SubTaskOrGroupTasks"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobSubTask"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"taskName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"state"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BackupTask"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"SubTaskOrGroupTasks"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobGroupTasks"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"groupName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProgressTask"}}]}},{"kind":"Field","alias":{"kind":"Name","value":"taskDescription"},"name":{"kind":"Name","value":"subtasks"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"TaskDescription"}}]}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobSubTask"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"taskName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProgressTask"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Job"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Job"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"queueName"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"failedReason"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"groupName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"ip"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobProgression"}}]}},{"kind":"Field","name":{"kind":"Name","value":"subtasks"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BackupTask"}}]}}]}}]}}]} as unknown as DocumentNode<QueueTasksJobUpdatedSubscription, QueueTasksJobUpdatedSubscriptionVariables>;
import { bigintTypePolicy } from '../utils/bigint.utils';

export const scalarTypePolicies = {
  Backup: {
    fields: {
      compressedFileSize: bigintTypePolicy,
      existingCompressedFileSize: bigintTypePolicy,
      existingFileSize: bigintTypePolicy,
      fileSize: bigintTypePolicy,
      newCompressedFileSize: bigintTypePolicy,
      newFileSize: bigintTypePolicy,
    },
  },
  BigIntTimeSerie: { fields: { value: bigintTypePolicy } },
  DiskUsage: {
    fields: {
      free: bigintTypePolicy,
      freeLastMonth: bigintTypePolicy,
      total: bigintTypePolicy,
      totalLastMonth: bigintTypePolicy,
      used: bigintTypePolicy,
      usedLastMonth: bigintTypePolicy,
    },
  },
  HostStatistics: {
    fields: {
      compressedSize: bigintTypePolicy,
      compressedSizeLastMonth: bigintTypePolicy,
      size: bigintTypePolicy,
      sizeLastMonth: bigintTypePolicy,
    },
  },
  JobProgression: {
    fields: {
      compressedFileSize: bigintTypePolicy,
      fileSize: bigintTypePolicy,
      newCompressedFileSize: bigintTypePolicy,
      newFileSize: bigintTypePolicy,
      progressCurrent: bigintTypePolicy,
      progressMax: bigintTypePolicy,
    },
  },
  PoolUsage: {
    fields: {
      compressedSize: bigintTypePolicy,
      compressedSizeLastMonth: bigintTypePolicy,
      size: bigintTypePolicy,
      sizeLastMonth: bigintTypePolicy,
      unusedSize: bigintTypePolicy,
      unusedSizeLastMonth: bigintTypePolicy,
    },
  },
};
