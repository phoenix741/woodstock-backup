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
  /** Date custom scalar type */
  BigInt: string;
};

export type Backup = {
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
  excludes?: Maybe<Array<Scalars['String']>>;
  includes?: Maybe<Array<Scalars['String']>>;
  name: Scalars['String'];
  shares: Array<BackupTaskShare>;
  timeout?: Maybe<Scalars['Float']>;
};

export enum BackupState {
  Aborted = 'ABORTED',
  Failed = 'FAILED',
  Running = 'RUNNING',
  Success = 'SUCCESS',
  Waiting = 'WAITING'
}

export type BackupSubTask = {
  context: Scalars['String'];
  description: Scalars['String'];
  progression?: Maybe<TaskProgression>;
  state: BackupState;
};

export type BackupTask = {
  complete?: Maybe<Scalars['Boolean']>;
  config?: Maybe<HostConfiguration>;
  host: Scalars['String'];
  ip?: Maybe<Scalars['String']>;
  number?: Maybe<Scalars['Float']>;
  originalStartDate?: Maybe<Scalars['Float']>;
  previousNumber?: Maybe<Scalars['Float']>;
  progression?: Maybe<TaskProgression>;
  startDate?: Maybe<Scalars['Float']>;
  state?: Maybe<BackupState>;
  subtasks?: Maybe<Array<BackupSubTask>>;
};

export type BackupTaskShare = {
  excludes?: Maybe<Array<Scalars['String']>>;
  includes?: Maybe<Array<Scalars['String']>>;
  name: Scalars['String'];
};

export type DhcpAddress = {
  address: Scalars['String'];
  end: Scalars['Float'];
  start: Scalars['Float'];
};

export type DiskUsage = {
  free?: Maybe<Scalars['Int']>;
  freeLastMonth?: Maybe<Scalars['Float']>;
  freeRange?: Maybe<Array<TimeSerie>>;
  total?: Maybe<Scalars['Int']>;
  totalLastMonth?: Maybe<Scalars['Float']>;
  totalRange?: Maybe<Array<TimeSerie>>;
  used?: Maybe<Scalars['Int']>;
  usedLastMonth?: Maybe<Scalars['Float']>;
  usedRange?: Maybe<Array<TimeSerie>>;
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
  command: Scalars['String'];
  name: Scalars['String'];
};

export type FileAcl = {
  group?: Maybe<Scalars['String']>;
  mask?: Maybe<Scalars['Float']>;
  other?: Maybe<Scalars['Float']>;
  user?: Maybe<Scalars['String']>;
};

export type FileDescription = {
  acl: Array<FileAcl>;
  path: Scalars['String'];
  stats?: Maybe<FileStat>;
  symlink?: Maybe<Scalars['String']>;
  type: EnumFileType;
};

export type FileStat = {
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
  backups: Array<Backup>;
  configuration: HostConfiguration;
  lastBackup?: Maybe<Backup>;
  lastBackupState?: Maybe<Scalars['String']>;
  name: Scalars['String'];
};

export type HostConfigOperation = {
  finalizeTasks?: Maybe<Array<Operation>>;
  tasks?: Maybe<Array<Operation>>;
};

export type HostConfiguration = {
  addresses?: Maybe<Array<Scalars['String']>>;
  dhcp?: Maybe<Array<DhcpAddress>>;
  isLocal?: Maybe<Scalars['Boolean']>;
  operations?: Maybe<HostConfigOperation>;
  schedule?: Maybe<Schedule>;
};

export type HostStatistics = {
  compressedSize?: Maybe<Scalars['Int']>;
  compressedSizeLastMonth?: Maybe<Scalars['Float']>;
  compressedSizeRange?: Maybe<Array<TimeSerie>>;
  host?: Maybe<Scalars['String']>;
  longestChain?: Maybe<Scalars['Int']>;
  longestChainLastMonth?: Maybe<Scalars['Float']>;
  longestChainRange?: Maybe<Array<TimeSerie>>;
  nbChunk?: Maybe<Scalars['Int']>;
  nbChunkLastMonth?: Maybe<Scalars['Float']>;
  nbChunkRange?: Maybe<Array<TimeSerie>>;
  nbRef?: Maybe<Scalars['Int']>;
  nbRefLastMonth?: Maybe<Scalars['Float']>;
  nbRefRange?: Maybe<Array<TimeSerie>>;
  size?: Maybe<Scalars['Int']>;
  sizeLastMonth?: Maybe<Scalars['Float']>;
  sizeRange?: Maybe<Array<TimeSerie>>;
};

export type Job = {
  attemptsMade: Scalars['Int'];
  data: BackupTask;
  delay: Scalars['Int'];
  failedReason?: Maybe<Scalars['String']>;
  finishedOn?: Maybe<Scalars['Int']>;
  id: Scalars['Int'];
  name: Scalars['String'];
  processedOn?: Maybe<Scalars['Int']>;
  progress?: Maybe<Scalars['Float']>;
  stacktrace?: Maybe<Array<Scalars['String']>>;
  state?: Maybe<Scalars['String']>;
  timestamp: Scalars['Int'];
};

export type JobResponse = {
  id: Scalars['Float'];
};

export type Mutation = {
  createBackup: JobResponse;
  removeBackup: JobResponse;
};


export type MutationCreateBackupArgs = {
  hostname: Scalars['String'];
};


export type MutationRemoveBackupArgs = {
  hostname: Scalars['String'];
  number: Scalars['Int'];
};

export type Operation = BackupOperation | ExecuteCommandOperation;

export type PoolUsage = {
  compressedSize?: Maybe<Scalars['Int']>;
  compressedSizeLastMonth?: Maybe<Scalars['Float']>;
  compressedSizeRange?: Maybe<Array<TimeSerie>>;
  longestChain?: Maybe<Scalars['Int']>;
  longestChainLastMonth?: Maybe<Scalars['Float']>;
  longestChainRange?: Maybe<Array<TimeSerie>>;
  nbChunk?: Maybe<Scalars['Int']>;
  nbChunkLastMonth?: Maybe<Scalars['Float']>;
  nbChunkRange?: Maybe<Array<TimeSerie>>;
  nbRef?: Maybe<Scalars['Int']>;
  nbRefLastMonth?: Maybe<Scalars['Float']>;
  nbRefRange?: Maybe<Array<TimeSerie>>;
  size?: Maybe<Scalars['Int']>;
  sizeLastMonth?: Maybe<Scalars['Float']>;
  sizeRange?: Maybe<Array<TimeSerie>>;
};

export type Query = {
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
  state?: InputMaybe<Array<Scalars['String']>>;
};

export type QueueStats = {
  active: Scalars['Int'];
  completed: Scalars['Int'];
  delayed: Scalars['Int'];
  failed: Scalars['Int'];
  lastExecution?: Maybe<Scalars['Float']>;
  nextWakeup?: Maybe<Scalars['Float']>;
  waiting: Scalars['Int'];
};

export type Schedule = {
  activated?: Maybe<Scalars['Boolean']>;
  backupPeriod?: Maybe<Scalars['Float']>;
  backupToKeep?: Maybe<ScheduledBackupToKeep>;
};

export type ScheduledBackupToKeep = {
  daily?: Maybe<Scalars['Float']>;
  hourly?: Maybe<Scalars['Float']>;
  monthly?: Maybe<Scalars['Float']>;
  weekly?: Maybe<Scalars['Float']>;
  yearly?: Maybe<Scalars['Float']>;
};

export type Statistics = {
  diskUsage?: Maybe<DiskUsage>;
  hosts?: Maybe<Array<HostStatistics>>;
  poolUsage?: Maybe<PoolUsage>;
};

export type Subscription = {
  jobFailed: Job;
  jobRemoved: Job;
  jobUpdated: Job;
  jobWaiting: Scalars['Int'];
};

export type TaskProgression = {
  compressedFileSize: Scalars['BigInt'];
  fileCount: Scalars['Float'];
  fileSize: Scalars['BigInt'];
  newCompressedFileSize: Scalars['BigInt'];
  newFileCount: Scalars['Float'];
  newFileSize: Scalars['BigInt'];
  percent: Scalars['Int'];
  progressCurrent: Scalars['BigInt'];
  progressMax: Scalars['BigInt'];
  speed: Scalars['Float'];
};

export type TimeSerie = {
  time: Scalars['BigInt'];
  value: Scalars['Float'];
};

export type BackupsQueryVariables = Exact<{
  hostname: Scalars['String'];
}>;


export type BackupsQuery = { backups: Array<{ number: number, complete: boolean, startDate: number, endDate?: number | null, fileCount: number, newFileCount: number, existingFileCount: number, fileSize: string, newFileSize: string, existingFileSize: string, compressedFileSize: string, newCompressedFileSize: string, existingCompressedFileSize: string, speed: number }> };

export type CreateBackupMutationVariables = Exact<{
  hostname: Scalars['String'];
}>;


export type CreateBackupMutation = { createBackup: { id: number } };

export type DashboardQueryVariables = Exact<{ [key: string]: never; }>;


export type DashboardQuery = { queueStats: { waiting: number, active: number, failed: number, lastExecution?: number | null, nextWakeup?: number | null }, statistics: { diskUsage?: { used?: number | null, usedLastMonth?: number | null, free?: number | null, total?: number | null } | null, poolUsage?: { size?: number | null, sizeLastMonth?: number | null, compressedSize?: number | null, compressedSizeLastMonth?: number | null, longestChain?: number | null, longestChainLastMonth?: number | null, nbChunk?: number | null, nbChunkLastMonth?: number | null, nbRef?: number | null, nbRefLastMonth?: number | null, sizeRange?: Array<{ time: string, value: number }> | null, compressedSizeRange?: Array<{ time: string, value: number }> | null, nbChunkRange?: Array<{ time: string, value: number }> | null } | null, hosts?: Array<{ host?: string | null, size?: number | null, compressedSize?: number | null }> | null } };

export type HostsQueryVariables = Exact<{ [key: string]: never; }>;


export type HostsQuery = { hosts: Array<{ name: string, lastBackupState?: string | null, lastBackup?: { number: number, startDate: number, fileSize: string, compressedFileSize: string, complete: boolean } | null, configuration: { schedule?: { activated?: boolean | null } | null } }>, statistics: { hosts?: Array<{ host?: string | null, size?: number | null, compressedSize?: number | null }> | null } };


export const BackupsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"Backups"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"backups"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"hostname"},"value":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"complete"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"endDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"existingFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"existingFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"existingCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}}]}}]}}]} as unknown as DocumentNode<BackupsQuery, BackupsQueryVariables>;
export const CreateBackupDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"createBackup"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"createBackup"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"hostname"},"value":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]} as unknown as DocumentNode<CreateBackupMutation, CreateBackupMutationVariables>;
export const DashboardDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"Dashboard"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"queueStats"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"waiting"}},{"kind":"Field","name":{"kind":"Name","value":"active"}},{"kind":"Field","name":{"kind":"Name","value":"failed"}},{"kind":"Field","name":{"kind":"Name","value":"lastExecution"}},{"kind":"Field","name":{"kind":"Name","value":"nextWakeup"}}]}},{"kind":"Field","name":{"kind":"Name","value":"statistics"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"diskUsage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"used"}},{"kind":"Field","name":{"kind":"Name","value":"usedLastMonth"}},{"kind":"Field","name":{"kind":"Name","value":"free"}},{"kind":"Field","name":{"kind":"Name","value":"total"}}]}},{"kind":"Field","name":{"kind":"Name","value":"poolUsage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"sizeLastMonth"}},{"kind":"Field","name":{"kind":"Name","value":"sizeRange"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"time"}},{"kind":"Field","name":{"kind":"Name","value":"value"}}]}},{"kind":"Field","name":{"kind":"Name","value":"compressedSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedSizeLastMonth"}},{"kind":"Field","name":{"kind":"Name","value":"compressedSizeRange"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"time"}},{"kind":"Field","name":{"kind":"Name","value":"value"}}]}},{"kind":"Field","name":{"kind":"Name","value":"longestChain"}},{"kind":"Field","name":{"kind":"Name","value":"longestChainLastMonth"}},{"kind":"Field","name":{"kind":"Name","value":"nbChunk"}},{"kind":"Field","name":{"kind":"Name","value":"nbChunkLastMonth"}},{"kind":"Field","name":{"kind":"Name","value":"nbChunkRange"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"time"}},{"kind":"Field","name":{"kind":"Name","value":"value"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nbRef"}},{"kind":"Field","name":{"kind":"Name","value":"nbRefLastMonth"}}]}},{"kind":"Field","name":{"kind":"Name","value":"hosts"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"compressedSize"}}]}}]}}]}}]} as unknown as DocumentNode<DashboardQuery, DashboardQueryVariables>;
export const HostsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"Hosts"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"hosts"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"lastBackup"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"complete"}}]}},{"kind":"Field","name":{"kind":"Name","value":"lastBackupState"}},{"kind":"Field","name":{"kind":"Name","value":"configuration"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"schedule"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"activated"}}]}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"statistics"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"hosts"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"compressedSize"}}]}}]}}]}}]} as unknown as DocumentNode<HostsQuery, HostsQueryVariables>;