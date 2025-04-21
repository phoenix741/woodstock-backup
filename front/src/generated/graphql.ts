/* eslint-disable */
import type { TypedDocumentNode as DocumentNode } from '@graphql-typed-document-node/core';
export type Maybe<T> = T | null;
export type InputMaybe<T> = Maybe<T>;
export type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };
export type MakeEmpty<T extends { [key: string]: unknown }, K extends keyof T> = { [_ in K]?: never };
export type Incremental<T> = T | { [P in keyof T]?: P extends ' $fragmentName' | '__typename' ? T[P] : never };
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: { input: string; output: string; }
  String: { input: string; output: string; }
  Boolean: { input: boolean; output: boolean; }
  Int: { input: number; output: number; }
  Float: { input: number; output: number; }
  /** The `BigInt` scalar type represents non-fractional signed whole numeric values. BigInt can represent values between -(2^63) + 1 and 2^63 - 1. */
  BigInt: { input: bigint; output: bigint; }
  /** A date-time string at UTC, such as 2019-12-03T09:54:33Z, compliant with the date-time format. */
  DateTime: { input: string; output: string; }
};

export type ApplicationEvent = {
  __typename?: 'ApplicationEvent';
  errorMessages: Array<Scalars['String']['output']>;
  information?: Maybe<EventInformation>;
  source: EventSource;
  status: EventStatus;
  step: EventStep;
  timestamp: Scalars['DateTime']['output'];
  type: EventType;
  uuid: Scalars['String']['output'];
};

export type Backup = {
  __typename?: 'Backup';
  agentVersion?: Maybe<Scalars['String']['output']>;
  compressedFileSize: Scalars['BigInt']['output'];
  endDate?: Maybe<Scalars['Float']['output']>;
  errorCount: Scalars['Float']['output'];
  existingCompressedFileSize: Scalars['BigInt']['output'];
  existingFileCount: Scalars['Float']['output'];
  existingFileSize: Scalars['BigInt']['output'];
  fileCount: Scalars['Float']['output'];
  fileSize: Scalars['BigInt']['output'];
  files: Array<FileDescription>;
  id: Scalars['ID']['output'];
  modifiedCompressedFileSize: Scalars['BigInt']['output'];
  modifiedFileCount: Scalars['Float']['output'];
  modifiedFileSize: Scalars['BigInt']['output'];
  newCompressedFileSize: Scalars['BigInt']['output'];
  newFileCount: Scalars['Float']['output'];
  newFileSize: Scalars['BigInt']['output'];
  number: Scalars['Float']['output'];
  removedFileCount: Scalars['Float']['output'];
  shares: Array<FileDescription>;
  speed: Scalars['Float']['output'];
  startDate: Scalars['Float']['output'];
  status: BackupStatus;
};


export type BackupFilesArgs = {
  path: Scalars['String']['input'];
  sharePath: Scalars['String']['input'];
};

export enum BackupErrorState {
  AddReferencesToPoolError = 'AddReferencesToPoolError',
  AuthenticationError = 'AuthenticationError',
  BackupError = 'BackupError',
  CommandExecutionError = 'CommandExecutionError',
  CompactError = 'CompactError',
  CountReferencesError = 'CountReferencesError',
  InitializationError = 'InitializationError',
  Unknown = 'Unknown'
}

export enum BackupExecutionState {
  AddReferencesToPool = 'AddReferencesToPool',
  Authenticate = 'Authenticate',
  Compact = 'Compact',
  Completed = 'Completed',
  CountReferences = 'CountReferences',
  DownloadChunks = 'DownloadChunks',
  DownloadFileList = 'DownloadFileList',
  Initialization = 'Initialization',
  PostCommands = 'PostCommands',
  PreCommands = 'PreCommands',
  Waiting = 'Waiting'
}

export type BackupOperation = {
  __typename?: 'BackupOperation';
  excludes?: Maybe<Array<Scalars['String']['output']>>;
  includes?: Maybe<Array<Scalars['String']['output']>>;
  shares: Array<BackupTaskShare>;
  timeout?: Maybe<Scalars['Float']['output']>;
};

export type BackupProgression = {
  __typename?: 'BackupProgression';
  compressedFileSize: Scalars['BigInt']['output'];
  endTransferDate?: Maybe<Scalars['Int']['output']>;
  errorCount: Scalars['Int']['output'];
  fileCount: Scalars['Int']['output'];
  fileSize: Scalars['BigInt']['output'];
  modifiedCompressedFileSize: Scalars['BigInt']['output'];
  modifiedFileCount: Scalars['Int']['output'];
  modifiedFileSize: Scalars['BigInt']['output'];
  newCompressedFileSize: Scalars['BigInt']['output'];
  newFileCount: Scalars['Int']['output'];
  newFileSize: Scalars['BigInt']['output'];
  percent: Scalars['Float']['output'];
  progressCurrent: Scalars['BigInt']['output'];
  progressMax: Scalars['BigInt']['output'];
  removedFileCount: Scalars['Int']['output'];
  speed: Scalars['Float']['output'];
  startDate: Scalars['Int']['output'];
  startTransferDate?: Maybe<Scalars['Int']['output']>;
};

export type BackupQueueData = JobBackupData | JobCleanupData | JobFsckData | JobRemoveData | JobRestoreData;

export enum BackupStatus {
  Aborted = 'Aborted',
  Completed = 'Completed',
  Failed = 'Failed',
  Finishing = 'Finishing',
  InProgress = 'InProgress'
}

/**
 * Part of config file.
 *
 * Store information about a share
 */
export type BackupTaskShare = {
  __typename?: 'BackupTaskShare';
  excludes?: Maybe<Array<Scalars['String']['output']>>;
  includes?: Maybe<Array<Scalars['String']['output']>>;
  name: Scalars['String']['output'];
};

export type BackupTaskState = {
  __typename?: 'BackupTaskState';
  errorMessage?: Maybe<Scalars['String']['output']>;
  errorState?: Maybe<BackupErrorState>;
  executionState: BackupExecutionState;
  postCommandStates: Array<ExecuteCommandState>;
  preCommandStates: Array<ExecuteCommandState>;
  progression: BackupProgression;
  shareStates: Array<ShareState>;
};

export type BigIntTimeSerie = {
  __typename?: 'BigIntTimeSerie';
  time: Scalars['Float']['output'];
  value: Scalars['BigInt']['output'];
};

export enum ChunkAlgorithm {
  Blake3 = 'Blake3',
  Sha2_256 = 'Sha2_256',
  Sha3_256 = 'Sha3_256'
}

export type ChunkProgression = {
  __typename?: 'ChunkProgression';
  errorCount: Scalars['Int']['output'];
  progressCurrent: Scalars['Int']['output'];
  progressMax: Scalars['Int']['output'];
  totalCount: Scalars['Int']['output'];
};

export enum CleanerErrorState {
  ApplyingRefcntError = 'ApplyingRefcntError',
  CleaningError = 'CleaningError',
  InitializationError = 'InitializationError',
  Unknown = 'Unknown'
}

export enum CleanerExecutionState {
  ApplyingRefcnt = 'ApplyingRefcnt',
  Cleaning = 'Cleaning',
  Completed = 'Completed',
  Initialization = 'Initialization',
  Waiting = 'Waiting'
}

export type CleanerProgression = {
  __typename?: 'CleanerProgression';
  compressedFileSize: Scalars['BigInt']['output'];
  fileSize: Scalars['BigInt']['output'];
  progressCurrent: Scalars['Int']['output'];
  progressMax: Scalars['Int']['output'];
};

export type CleanerTaskState = {
  __typename?: 'CleanerTaskState';
  errorMessage?: Maybe<Scalars['String']['output']>;
  errorState?: Maybe<CleanerErrorState>;
  executionState: CleanerExecutionState;
  progression: CleanerProgression;
};

export type ClearCacheResponse = {
  __typename?: 'ClearCacheResponse';
  void?: Maybe<Scalars['Float']['output']>;
};

export type DiskUsage = {
  __typename?: 'DiskUsage';
  free?: Maybe<Scalars['BigInt']['output']>;
  freeLastMonth?: Maybe<Scalars['BigInt']['output']>;
  freeRange?: Maybe<Array<BigIntTimeSerie>>;
  total?: Maybe<Scalars['BigInt']['output']>;
  totalLastMonth?: Maybe<Scalars['BigInt']['output']>;
  totalRange?: Maybe<Array<BigIntTimeSerie>>;
  used?: Maybe<Scalars['BigInt']['output']>;
  usedLastMonth?: Maybe<Scalars['BigInt']['output']>;
  usedRange?: Maybe<Array<BigIntTimeSerie>>;
};

export enum EnumFileType {
  BlockDevice = 'BlockDevice',
  CharacterDevice = 'CharacterDevice',
  Directory = 'Directory',
  Fifo = 'Fifo',
  RegularFile = 'RegularFile',
  Socket = 'Socket',
  Symlink = 'Symlink',
  Unknown = 'Unknown'
}

export type EventBackupInformation = {
  __typename?: 'EventBackupInformation';
  hostname: Scalars['String']['output'];
  number: Scalars['Int']['output'];
  sharePath: Array<Scalars['String']['output']>;
};

export type EventHashConversionInformation = {
  __typename?: 'EventHashConversionInformation';
  algorithm: ChunkAlgorithm;
  count: Scalars['Int']['output'];
};

export type EventInformation = EventBackupInformation | EventHashConversionInformation | EventPoolCleanedInformation | EventPoolInformation;

export type EventPoolCleanedInformation = {
  __typename?: 'EventPoolCleanedInformation';
  count: Scalars['Int']['output'];
  size: Scalars['BigInt']['output'];
};

export type EventPoolInformation = {
  __typename?: 'EventPoolInformation';
  chunkCount: Scalars['Int']['output'];
  chunkError: Scalars['Int']['output'];
  fix: Scalars['Boolean']['output'];
  inNothing: Scalars['Int']['output'];
  inRefcnt: Scalars['Int']['output'];
  inUnused: Scalars['Int']['output'];
  missing: Scalars['Int']['output'];
  refcount: Scalars['Int']['output'];
  refcountError: Scalars['Int']['output'];
};

export enum EventSource {
  Cli = 'Cli',
  Import = 'Import',
  User = 'User',
  Woodstock = 'Woodstock'
}

export enum EventStatus {
  ClientDisconnected = 'ClientDisconnected',
  GenericError = 'GenericError',
  None = 'None',
  ServerCrashed = 'ServerCrashed',
  Success = 'Success'
}

export enum EventStep {
  End = 'End',
  Start = 'Start'
}

export enum EventType {
  Backup = 'Backup',
  Delete = 'Delete',
  HashConversion = 'HashConversion',
  PoolChecked = 'PoolChecked',
  PoolCleaned = 'PoolCleaned',
  Restore = 'Restore'
}

export enum ExecuteCommandExecutionState {
  Failed = 'Failed',
  InProgress = 'InProgress',
  Success = 'Success',
  Waiting = 'Waiting'
}

export type ExecuteCommandOperation = {
  __typename?: 'ExecuteCommandOperation';
  command: Scalars['String']['output'];
};

export type ExecuteCommandState = {
  __typename?: 'ExecuteCommandState';
  command: ExecuteCommandOperation;
  executionState: ExecuteCommandExecutionState;
};

export type FileAcl = {
  __typename?: 'FileAcl';
  id: Scalars['Int']['output'];
  perm: Scalars['Int']['output'];
  qualifier: FileManifestAclQualifier;
};

export type FileDescription = {
  __typename?: 'FileDescription';
  acl: Array<FileAcl>;
  path: Scalars['String']['output'];
  stats?: Maybe<FileStat>;
  symlink: Scalars['String']['output'];
  type: EnumFileType;
  xattr: Array<FileXattr>;
};

export type FileListProgression = {
  __typename?: 'FileListProgression';
  fileSize: Scalars['BigInt']['output'];
  modifiedFileCount: Scalars['Int']['output'];
  modifiedFileSize: Scalars['BigInt']['output'];
  newFileCount: Scalars['Int']['output'];
  newFileSize: Scalars['BigInt']['output'];
  removedFileCount: Scalars['Int']['output'];
};

export enum FileManifestAclQualifier {
  GroupId = 'GroupId',
  GroupObj = 'GroupObj',
  Mask = 'Mask',
  Other = 'Other',
  Undefined = 'Undefined',
  UserId = 'UserId',
  UserObj = 'UserObj'
}

export type FileStat = {
  __typename?: 'FileStat';
  compressedSize: Scalars['String']['output'];
  created: Scalars['String']['output'];
  dev: Scalars['BigInt']['output'];
  groupId: Scalars['Int']['output'];
  ino: Scalars['BigInt']['output'];
  lastModified: Scalars['String']['output'];
  lastRead: Scalars['String']['output'];
  mode: Scalars['Int']['output'];
  nlink: Scalars['BigInt']['output'];
  ownerId: Scalars['Int']['output'];
  rdev: Scalars['BigInt']['output'];
  size: Scalars['String']['output'];
  type: EnumFileType;
};

export type FileXattr = {
  __typename?: 'FileXattr';
  key: Scalars['String']['output'];
  value: Scalars['String']['output'];
};

export enum FsckErrorState {
  ApplyingRefcntError = 'ApplyingRefcntError',
  InitializationError = 'InitializationError',
  Unknown = 'Unknown',
  VerifyChunkError = 'VerifyChunkError',
  VerifyRefcntError = 'VerifyRefcntError',
  VerifyUnusedError = 'VerifyUnusedError'
}

export enum FsckExecutionState {
  ApplyingRefcnt = 'ApplyingRefcnt',
  Completed = 'Completed',
  Initialization = 'Initialization',
  VerifyChunk = 'VerifyChunk',
  VerifyRefcnt = 'VerifyRefcnt',
  VerifyUnused = 'VerifyUnused',
  Waiting = 'Waiting'
}

export type FsckTaskState = {
  __typename?: 'FsckTaskState';
  chunkProgression: ChunkProgression;
  dryRun: Scalars['Boolean']['output'];
  errorMessage?: Maybe<Scalars['String']['output']>;
  errorState?: Maybe<FsckErrorState>;
  executionState: FsckExecutionState;
  refcntProgression: RefcntProgression;
  unusedProgression: UnusedProgression;
};

export type Host = {
  __typename?: 'Host';
  addresses?: Maybe<Array<Scalars['String']['output']>>;
  agentVersion?: Maybe<Scalars['String']['output']>;
  availibilityState?: Maybe<HostAvailibilityState>;
  backups: Array<Backup>;
  configuration: HostConfiguration;
  dateToNextBackup?: Maybe<Scalars['DateTime']['output']>;
  lastBackup?: Maybe<Backup>;
  name: Scalars['ID']['output'];
  timeSinceLastBackup?: Maybe<Scalars['Float']['output']>;
  timeToNextBackup?: Maybe<Scalars['Float']['output']>;
};

export enum HostAvailibilityState {
  Offline = 'Offline',
  Online = 'Online',
  Unknown = 'Unknown'
}

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
  addresses?: Maybe<Array<Scalars['String']['output']>>;
  operations: HostConfigOperation;
  password: Scalars['String']['output'];
  port: Scalars['Float']['output'];
  schedule?: Maybe<Schedule>;
};

export type HostStatistics = {
  __typename?: 'HostStatistics';
  compressedSize?: Maybe<Scalars['BigInt']['output']>;
  compressedSizeLastMonth?: Maybe<Scalars['BigInt']['output']>;
  compressedSizeRange?: Maybe<Array<BigIntTimeSerie>>;
  host?: Maybe<Scalars['String']['output']>;
  longestChain?: Maybe<Scalars['Int']['output']>;
  longestChainLastMonth?: Maybe<Scalars['Int']['output']>;
  longestChainRange?: Maybe<Array<NumberTimeSerie>>;
  nbChunk?: Maybe<Scalars['Int']['output']>;
  nbChunkLastMonth?: Maybe<Scalars['Int']['output']>;
  nbChunkRange?: Maybe<Array<NumberTimeSerie>>;
  nbRef?: Maybe<Scalars['Int']['output']>;
  nbRefLastMonth?: Maybe<Scalars['Int']['output']>;
  nbRefRange?: Maybe<Array<NumberTimeSerie>>;
  size?: Maybe<Scalars['BigInt']['output']>;
  sizeLastMonth?: Maybe<Scalars['BigInt']['output']>;
  sizeRange?: Maybe<Array<BigIntTimeSerie>>;
};

export type Job = {
  __typename?: 'Job';
  attemptsMade: Scalars['Int']['output'];
  data?: Maybe<BackupQueueData>;
  failedReason?: Maybe<Scalars['String']['output']>;
  id?: Maybe<Scalars['String']['output']>;
  name: Scalars['String']['output'];
  progression?: Maybe<TaskState>;
  queueName: Scalars['String']['output'];
  state: Scalars['String']['output'];
};

export type JobBackupData = {
  __typename?: 'JobBackupData';
  config?: Maybe<HostConfiguration>;
  force?: Maybe<Scalars['Boolean']['output']>;
  host: Scalars['String']['output'];
  ip?: Maybe<Scalars['String']['output']>;
  number?: Maybe<Scalars['Float']['output']>;
  previousNumber?: Maybe<Scalars['Float']['output']>;
  startDate?: Maybe<Scalars['Float']['output']>;
};

export type JobCleanupData = {
  __typename?: 'JobCleanupData';
  target?: Maybe<Scalars['String']['output']>;
};

export type JobFsckData = {
  __typename?: 'JobFsckData';
  dryRun: Scalars['Boolean']['output'];
  verifyChunks: Scalars['Boolean']['output'];
};

export type JobRemoveData = {
  __typename?: 'JobRemoveData';
  config?: Maybe<HostConfiguration>;
  host: Scalars['String']['output'];
  number?: Maybe<Scalars['Float']['output']>;
  startDate?: Maybe<Scalars['Float']['output']>;
};

export type JobResponse = {
  __typename?: 'JobResponse';
  id: Scalars['String']['output'];
};

export type JobRestoreData = {
  __typename?: 'JobRestoreData';
  config?: Maybe<HostConfiguration>;
  destinationDirectory: Scalars['String']['output'];
  files: Array<JobRestoreDataSelection>;
  host: Scalars['String']['output'];
  ip?: Maybe<Scalars['String']['output']>;
  number?: Maybe<Scalars['Float']['output']>;
  startDate?: Maybe<Scalars['Float']['output']>;
};

export type JobRestoreDataSelection = {
  __typename?: 'JobRestoreDataSelection';
  selection: Array<Scalars['String']['output']>;
  share: Scalars['String']['output'];
};

export type Mutation = {
  __typename?: 'Mutation';
  checkAndFixPool: JobResponse;
  cleanupPool: JobResponse;
  clearCache: ClearCacheResponse;
  createBackup: JobResponse;
  removeBackup: JobResponse;
  restoreBackup: JobResponse;
};


export type MutationCheckAndFixPoolArgs = {
  fix: Scalars['Boolean']['input'];
  verifyChunks: Scalars['Boolean']['input'];
};


export type MutationCreateBackupArgs = {
  hostname: Scalars['String']['input'];
};


export type MutationRemoveBackupArgs = {
  hostname: Scalars['String']['input'];
  number: Scalars['Int']['input'];
};


export type MutationRestoreBackupArgs = {
  input: RestoreInput;
};

export type NumberTimeSerie = {
  __typename?: 'NumberTimeSerie';
  time: Scalars['Float']['output'];
  value: Scalars['Int']['output'];
};

export type PoolUsage = {
  __typename?: 'PoolUsage';
  compressedSize?: Maybe<Scalars['BigInt']['output']>;
  compressedSizeLastMonth?: Maybe<Scalars['BigInt']['output']>;
  compressedSizeRange?: Maybe<Array<BigIntTimeSerie>>;
  longestChain?: Maybe<Scalars['Int']['output']>;
  longestChainLastMonth?: Maybe<Scalars['Int']['output']>;
  longestChainRange?: Maybe<Array<NumberTimeSerie>>;
  nbChunk?: Maybe<Scalars['Int']['output']>;
  nbChunkLastMonth?: Maybe<Scalars['Int']['output']>;
  nbChunkRange?: Maybe<Array<NumberTimeSerie>>;
  nbRef?: Maybe<Scalars['Int']['output']>;
  nbRefLastMonth?: Maybe<Scalars['Int']['output']>;
  nbRefRange?: Maybe<Array<NumberTimeSerie>>;
  size?: Maybe<Scalars['BigInt']['output']>;
  sizeLastMonth?: Maybe<Scalars['BigInt']['output']>;
  sizeRange?: Maybe<Array<BigIntTimeSerie>>;
  unusedSize?: Maybe<Scalars['BigInt']['output']>;
  unusedSizeLastMonth?: Maybe<Scalars['BigInt']['output']>;
  unusedSizeRange?: Maybe<Array<BigIntTimeSerie>>;
};

export type Query = {
  __typename?: 'Query';
  backup: Backup;
  backups: Array<Backup>;
  events: Array<ApplicationEvent>;
  host: Host;
  hosts: Array<Host>;
  informations: ServerInformations;
  queue: Array<Job>;
  queueStats: QueueStats;
  statistics: Statistics;
};


export type QueryBackupArgs = {
  hostname: Scalars['String']['input'];
  number: Scalars['Int']['input'];
};


export type QueryBackupsArgs = {
  hostname: Scalars['String']['input'];
};


export type QueryEventsArgs = {
  firstEvent: Scalars['DateTime']['input'];
  lastEvent: Scalars['DateTime']['input'];
};


export type QueryHostArgs = {
  hostname: Scalars['String']['input'];
};


export type QueryQueueArgs = {
  input: QueueListInput;
};

export type QueueListInput = {
  operationName?: InputMaybe<Scalars['String']['input']>;
  queueName?: InputMaybe<Scalars['String']['input']>;
  states?: Array<Scalars['String']['input']>;
};

export type QueueStats = {
  __typename?: 'QueueStats';
  active: Scalars['Int']['output'];
  completed: Scalars['Int']['output'];
  delayed: Scalars['Int']['output'];
  failed: Scalars['Int']['output'];
  lastExecution?: Maybe<Scalars['Float']['output']>;
  nextWakeup?: Maybe<Scalars['Float']['output']>;
  waiting: Scalars['Int']['output'];
  waitingChildren: Scalars['Int']['output'];
};

export type RefcntProgression = {
  __typename?: 'RefcntProgression';
  errorCount: Scalars['Int']['output'];
  progressCurrent: Scalars['Int']['output'];
  progressMax: Scalars['Int']['output'];
  totalCount: Scalars['Int']['output'];
};

export enum RemoveErrorState {
  AddReferencesToPoolError = 'AddReferencesToPoolError',
  BackupRemovalError = 'BackupRemovalError',
  RefcntRemovalError = 'RefcntRemovalError',
  Unknown = 'Unknown'
}

export enum RemoveExecutionState {
  AddReferencesToPool = 'AddReferencesToPool',
  Completed = 'Completed',
  RemovingBackup = 'RemovingBackup',
  RemovingRefCnt = 'RemovingRefCnt',
  Waiting = 'Waiting'
}

export type RemoveTaskState = {
  __typename?: 'RemoveTaskState';
  errorMessage?: Maybe<Scalars['String']['output']>;
  errorState?: Maybe<RemoveErrorState>;
  executionState: RemoveExecutionState;
};

export enum RestoreErrorState {
  AuthenticationError = 'AuthenticationError',
  PreparationError = 'PreparationError',
  RestoreError = 'RestoreError',
  Unknown = 'Unknown'
}

export enum RestoreExecutionState {
  Authentication = 'Authentication',
  Completed = 'Completed',
  Preparation = 'Preparation',
  Restoring = 'Restoring',
  Waiting = 'Waiting'
}

export type RestoreFilesInput = {
  selection: Array<Scalars['String']['input']>;
  share: Scalars['String']['input'];
};

export type RestoreInput = {
  destinationDirectory: Scalars['String']['input'];
  files: Array<RestoreFilesInput>;
  hostname: Scalars['String']['input'];
  number: Scalars['Int']['input'];
};

export type RestoreTaskState = {
  __typename?: 'RestoreTaskState';
  errorMessage?: Maybe<Scalars['String']['output']>;
  errorState?: Maybe<RestoreErrorState>;
  executionState: RestoreExecutionState;
  globalProgression: BackupProgression;
};

export type Schedule = {
  __typename?: 'Schedule';
  activated?: Maybe<Scalars['Boolean']['output']>;
  backupPeriod?: Maybe<Scalars['Float']['output']>;
  backupToKeep?: Maybe<ScheduledBackupToKeep>;
};

export type ScheduledBackupToKeep = {
  __typename?: 'ScheduledBackupToKeep';
  daily?: Maybe<Scalars['Float']['output']>;
  hourly?: Maybe<Scalars['Float']['output']>;
  monthly?: Maybe<Scalars['Float']['output']>;
  weekly?: Maybe<Scalars['Float']['output']>;
  yearly?: Maybe<Scalars['Float']['output']>;
};

export type ServerInformations = {
  __typename?: 'ServerInformations';
  hostname: Scalars['String']['output'];
  platform: Scalars['String']['output'];
  uptime: Scalars['Float']['output'];
  woodstockVersion?: Maybe<Scalars['String']['output']>;
};

export enum ShareExecutionState {
  Failed = 'Failed',
  FileList = 'FileList',
  InProgress = 'InProgress',
  Success = 'Success',
  Waiting = 'Waiting'
}

export type ShareState = {
  __typename?: 'ShareState';
  backupProgression: BackupProgression;
  executionState: ShareExecutionState;
  fileListProgression: FileListProgression;
  share: Scalars['String']['output'];
};

export type Statistics = {
  __typename?: 'Statistics';
  diskUsage?: Maybe<DiskUsage>;
  hosts?: Maybe<Array<HostStatistics>>;
  poolUsage?: Maybe<PoolUsage>;
};

export type Subscription = {
  __typename?: 'Subscription';
  jobFailed: Job;
  jobRemoved: Job;
  jobUpdated: Job;
  jobWaiting: Scalars['Int']['output'];
};

export type TaskState = BackupTaskState | CleanerTaskState | FsckTaskState | RemoveTaskState | RestoreTaskState;

export type UnusedProgression = {
  __typename?: 'UnusedProgression';
  inNothing: Scalars['Int']['output'];
  inRefcnt: Scalars['Int']['output'];
  inUnused: Scalars['Int']['output'];
  missing: Scalars['Int']['output'];
  progressCurrent: Scalars['Int']['output'];
  progressMax: Scalars['Int']['output'];
};

export type ApplicationEventFragment = { __typename?: 'ApplicationEvent', uuid: string, type: EventType, step: EventStep, source: EventSource, timestamp: string, errorMessages: Array<string>, status: EventStatus, information?: (
    { __typename: 'EventBackupInformation' }
    & { ' $fragmentRefs'?: { 'EventBackupInformationFragment': EventBackupInformationFragment } }
  ) | (
    { __typename: 'EventHashConversionInformation' }
    & { ' $fragmentRefs'?: { 'EventHashConversionInformationFragment': EventHashConversionInformationFragment } }
  ) | (
    { __typename: 'EventPoolCleanedInformation' }
    & { ' $fragmentRefs'?: { 'EventPoolCleanedInformationFragment': EventPoolCleanedInformationFragment } }
  ) | (
    { __typename: 'EventPoolInformation' }
    & { ' $fragmentRefs'?: { 'EventPoolInformationFragment': EventPoolInformationFragment } }
  ) | null } & { ' $fragmentName'?: 'ApplicationEventFragment' };

export type EventBackupInformationFragment = { __typename?: 'EventBackupInformation', hostname: string, number: number, sharePath: Array<string> } & { ' $fragmentName'?: 'EventBackupInformationFragment' };

export type EventPoolInformationFragment = { __typename?: 'EventPoolInformation', fix: boolean, refcount: number, refcountError: number, inUnused: number, inRefcnt: number, inNothing: number, missing: number, chunkCount: number, chunkError: number } & { ' $fragmentName'?: 'EventPoolInformationFragment' };

export type EventPoolCleanedInformationFragment = { __typename?: 'EventPoolCleanedInformation', size: bigint, count: number } & { ' $fragmentName'?: 'EventPoolCleanedInformationFragment' };

export type EventHashConversionInformationFragment = { __typename?: 'EventHashConversionInformation', count: number, algorithm: ChunkAlgorithm } & { ' $fragmentName'?: 'EventHashConversionInformationFragment' };

export type HostQueryVariables = Exact<{
  hostname: Scalars['String']['input'];
}>;


export type HostQuery = { __typename?: 'Query', host: { __typename?: 'Host', name: string, agentVersion?: string | null, availibilityState?: HostAvailibilityState | null, timeSinceLastBackup?: number | null, dateToNextBackup?: string | null, addresses?: Array<string> | null, lastBackup?: { __typename?: 'Backup', agentVersion?: string | null, status: BackupStatus } | null, configuration: { __typename?: 'HostConfiguration', operations: { __typename?: 'HostConfigOperation', preCommands?: Array<{ __typename?: 'ExecuteCommandOperation', command: string }> | null, operation?: { __typename?: 'BackupOperation', shares: Array<{ __typename?: 'BackupTaskShare', name: string }> } | null, postCommands?: Array<{ __typename?: 'ExecuteCommandOperation', command: string }> | null }, schedule?: { __typename?: 'Schedule', activated?: boolean | null } | null } } };

export type HostsQueryVariables = Exact<{ [key: string]: never; }>;


export type HostsQuery = { __typename?: 'Query', hosts: Array<{ __typename?: 'Host', name: string, agentVersion?: string | null, availibilityState?: HostAvailibilityState | null, timeSinceLastBackup?: number | null, dateToNextBackup?: string | null, lastBackup?: { __typename?: 'Backup', number: number, startDate: number, fileSize: bigint, status: BackupStatus, agentVersion?: string | null } | null, configuration: { __typename?: 'HostConfiguration', schedule?: { __typename?: 'Schedule', activated?: boolean | null } | null } }> };

export type BackupQueryVariables = Exact<{
  hostname: Scalars['String']['input'];
  number: Scalars['Int']['input'];
}>;


export type BackupQuery = { __typename?: 'Query', backup: { __typename?: 'Backup', id: string, number: number, status: BackupStatus, startDate: number, endDate?: number | null, errorCount: number, fileCount: number, newFileCount: number, existingFileCount: number, removedFileCount: number, modifiedFileCount: number, fileSize: bigint, newFileSize: bigint, existingFileSize: bigint, speed: number } };

export type BackupsQueryVariables = Exact<{
  hostname: Scalars['String']['input'];
}>;


export type BackupsQuery = { __typename?: 'Query', backups: Array<{ __typename?: 'Backup', id: string, number: number, status: BackupStatus, startDate: number, endDate?: number | null, errorCount: number, fileCount: number, newFileCount: number, existingFileCount: number, removedFileCount: number, modifiedFileCount: number, fileSize: bigint, newFileSize: bigint, existingFileSize: bigint, speed: number }> };

export type BackupsBrowseQueryVariables = Exact<{
  hostname: Scalars['String']['input'];
  number: Scalars['Int']['input'];
  sharePath: Scalars['String']['input'];
  path: Scalars['String']['input'];
}>;


export type BackupsBrowseQuery = { __typename?: 'Query', backup: { __typename?: 'Backup', id: string, files: Array<(
      { __typename?: 'FileDescription' }
      & { ' $fragmentRefs'?: { 'FragmentFileDescriptionFragment': FragmentFileDescriptionFragment } }
    )> } };

export type CreateBackupMutationVariables = Exact<{
  hostname: Scalars['String']['input'];
}>;


export type CreateBackupMutation = { __typename?: 'Mutation', createBackup: { __typename?: 'JobResponse', id: string } };

export type RemoveBackupMutationVariables = Exact<{
  hostname: Scalars['String']['input'];
  number: Scalars['Int']['input'];
}>;


export type RemoveBackupMutation = { __typename?: 'Mutation', removeBackup: { __typename?: 'JobResponse', id: string } };

export type FragmentFileDescriptionFragment = { __typename?: 'FileDescription', path: string, type: EnumFileType, symlink: string, stats?: { __typename?: 'FileStat', ownerId: number, groupId: number, mode: number, size: string, lastModified: string } | null } & { ' $fragmentName'?: 'FragmentFileDescriptionFragment' };

export type SharesBrowseQueryVariables = Exact<{
  hostname: Scalars['String']['input'];
  number: Scalars['Int']['input'];
}>;


export type SharesBrowseQuery = { __typename?: 'Query', backup: { __typename?: 'Backup', id: string, shares: Array<(
      { __typename?: 'FileDescription' }
      & { ' $fragmentRefs'?: { 'FragmentFileDescriptionFragment': FragmentFileDescriptionFragment } }
    )> } };

export type JobPoolResponseFragment = { __typename?: 'JobResponse', id: string } & { ' $fragmentName'?: 'JobPoolResponseFragment' };

export type RestoreBackupMutationVariables = Exact<{
  input: RestoreInput;
}>;


export type RestoreBackupMutation = { __typename?: 'Mutation', restoreBackup: (
    { __typename?: 'JobResponse' }
    & { ' $fragmentRefs'?: { 'JobPoolResponseFragment': JobPoolResponseFragment } }
  ) };

export type ClearCacheMutationVariables = Exact<{ [key: string]: never; }>;


export type ClearCacheMutation = { __typename?: 'Mutation', clearCache: { __typename?: 'ClearCacheResponse', void?: number | null } };

export type EventsQueryVariables = Exact<{
  firstEvent: Scalars['DateTime']['input'];
  lastEvent: Scalars['DateTime']['input'];
}>;


export type EventsQuery = { __typename?: 'Query', events: Array<(
    { __typename?: 'ApplicationEvent' }
    & { ' $fragmentRefs'?: { 'ApplicationEventFragment': ApplicationEventFragment } }
  )> };

export type CleanupPoolMutationVariables = Exact<{ [key: string]: never; }>;


export type CleanupPoolMutation = { __typename?: 'Mutation', cleanupPool: (
    { __typename?: 'JobResponse' }
    & { ' $fragmentRefs'?: { 'JobPoolResponseFragment': JobPoolResponseFragment } }
  ) };

export type CheckAndFixPoolMutationVariables = Exact<{
  fix: Scalars['Boolean']['input'];
  verifyChunks: Scalars['Boolean']['input'];
}>;


export type CheckAndFixPoolMutation = { __typename?: 'Mutation', checkAndFixPool: (
    { __typename?: 'JobResponse' }
    & { ' $fragmentRefs'?: { 'JobPoolResponseFragment': JobPoolResponseFragment } }
  ) };

export type ServerInformationsQueryVariables = Exact<{ [key: string]: never; }>;


export type ServerInformationsQuery = { __typename?: 'Query', informations: { __typename?: 'ServerInformations', platform: string, uptime: number, hostname: string, woodstockVersion?: string | null } };

export type DiskUsageStatisticsQueryVariables = Exact<{ [key: string]: never; }>;


export type DiskUsageStatisticsQuery = { __typename?: 'Query', statistics: { __typename?: 'Statistics', hosts?: Array<{ __typename?: 'HostStatistics', host?: string | null, size?: bigint | null, compressedSize?: bigint | null }> | null } };

export type QueueStatisticsQueryVariables = Exact<{ [key: string]: never; }>;


export type QueueStatisticsQuery = { __typename?: 'Query', queueStats: { __typename?: 'QueueStats', active: number, waiting: number, failed: number, delayed: number, completed: number } };

export type PoolStatisticsQueryVariables = Exact<{ [key: string]: never; }>;


export type PoolStatisticsQuery = { __typename?: 'Query', statistics: { __typename?: 'Statistics', diskUsage?: { __typename?: 'DiskUsage', used?: bigint | null, usedLastMonth?: bigint | null, free?: bigint | null, total?: bigint | null } | null, poolUsage?: { __typename?: 'PoolUsage', nbChunk?: number | null, nbChunkLastMonth?: number | null, nbRef?: number | null, nbRefLastMonth?: number | null, size?: bigint | null, compressedSize?: bigint | null, compressedSizeLastMonth?: bigint | null, unusedSize?: bigint | null, nbChunkRange?: Array<{ __typename?: 'NumberTimeSerie', time: number, value: number }> | null, compressedSizeRange?: Array<{ __typename?: 'BigIntTimeSerie', time: number, value: bigint }> | null } | null } };

export type JobBackupDataFragment = { __typename?: 'JobBackupData', host: string, number?: number | null, ip?: string | null, startDate?: number | null } & { ' $fragmentName'?: 'JobBackupDataFragment' };

export type JobRestoreDataFragment = { __typename?: 'JobRestoreData', host: string, number?: number | null, ip?: string | null, startDate?: number | null, destinationDirectory: string, files: Array<{ __typename?: 'JobRestoreDataSelection', share: string, selection: Array<string> }> } & { ' $fragmentName'?: 'JobRestoreDataFragment' };

export type JobRemoveDataFragment = { __typename?: 'JobRemoveData', host: string, number?: number | null, startDate?: number | null } & { ' $fragmentName'?: 'JobRemoveDataFragment' };

export type JobCleanupDataFragment = { __typename?: 'JobCleanupData', target?: string | null } & { ' $fragmentName'?: 'JobCleanupDataFragment' };

export type JobFsckDataFragment = { __typename?: 'JobFsckData', dryRun: boolean, verifyChunks: boolean } & { ' $fragmentName'?: 'JobFsckDataFragment' };

export type BackupTaskStateFragment = { __typename?: 'BackupTaskState', backupExecutionState: BackupExecutionState, backupErrorState?: BackupErrorState | null, backupErrorMessage?: string | null, backupProgress: { __typename?: 'BackupProgression', startDate: number, startTransferDate?: number | null, endTransferDate?: number | null, fileSize: bigint, newFileSize: bigint, modifiedFileSize: bigint, compressedFileSize: bigint, newCompressedFileSize: bigint, modifiedCompressedFileSize: bigint, fileCount: number, newFileCount: number, modifiedFileCount: number, removedFileCount: number, errorCount: number, speed: number, percent: number, progressCurrent: bigint, progressMax: bigint }, preCommandStates: Array<{ __typename?: 'ExecuteCommandState', executionState: ExecuteCommandExecutionState, command: { __typename?: 'ExecuteCommandOperation', command: string } }>, shareStates: Array<{ __typename?: 'ShareState', share: string, executionState: ShareExecutionState, backupProgression: { __typename?: 'BackupProgression', startDate: number, startTransferDate?: number | null, endTransferDate?: number | null, fileSize: bigint, newFileSize: bigint, modifiedFileSize: bigint, compressedFileSize: bigint, newCompressedFileSize: bigint, modifiedCompressedFileSize: bigint, fileCount: number, newFileCount: number, modifiedFileCount: number, removedFileCount: number, errorCount: number, speed: number, percent: number, progressCurrent: bigint, progressMax: bigint }, fileListProgression: { __typename?: 'FileListProgression', fileSize: bigint, newFileSize: bigint, modifiedFileSize: bigint, newFileCount: number, modifiedFileCount: number, removedFileCount: number } }>, postCommandStates: Array<{ __typename?: 'ExecuteCommandState', executionState: ExecuteCommandExecutionState, command: { __typename?: 'ExecuteCommandOperation', command: string } }> } & { ' $fragmentName'?: 'BackupTaskStateFragment' };

export type RestoreTaskStateFragment = { __typename?: 'RestoreTaskState', restoreExecutionState: RestoreExecutionState, restoreErrorState?: RestoreErrorState | null, restoreErrorMessage?: string | null, restoreProgression: { __typename?: 'BackupProgression', startDate: number, startTransferDate?: number | null, endTransferDate?: number | null, fileSize: bigint, newFileSize: bigint, modifiedFileSize: bigint, compressedFileSize: bigint, newCompressedFileSize: bigint, modifiedCompressedFileSize: bigint, fileCount: number, newFileCount: number, modifiedFileCount: number, removedFileCount: number, errorCount: number, speed: number, percent: number, progressCurrent: bigint, progressMax: bigint } } & { ' $fragmentName'?: 'RestoreTaskStateFragment' };

export type RemoveTaskStateFragment = { __typename?: 'RemoveTaskState', removeExecutionState: RemoveExecutionState, removeErrorState?: RemoveErrorState | null, removeErrorMessage?: string | null } & { ' $fragmentName'?: 'RemoveTaskStateFragment' };

export type CleanerTaskStateFragment = { __typename?: 'CleanerTaskState', cleanerExecutionState: CleanerExecutionState, cleanerErrorState?: CleanerErrorState | null, cleanerErrorMessage?: string | null, cleanerProgress: { __typename?: 'CleanerProgression', progressMax: number, progressCurrent: number, fileSize: bigint, compressedFileSize: bigint } } & { ' $fragmentName'?: 'CleanerTaskStateFragment' };

export type FsckTaskStateFragment = { __typename?: 'FsckTaskState', dryRun: boolean, fsckExecutionState: FsckExecutionState, fsckErrorState?: FsckErrorState | null, fsckErrorMessage?: string | null, refcntProgression: { __typename?: 'RefcntProgression', progressMax: number, progressCurrent: number, errorCount: number, totalCount: number }, unusedProgression: { __typename?: 'UnusedProgression', progressMax: number, progressCurrent: number, inNothing: number, inRefcnt: number, inUnused: number, missing: number }, chunkProgression: { __typename?: 'ChunkProgression', progressMax: number, progressCurrent: number, errorCount: number, totalCount: number } } & { ' $fragmentName'?: 'FsckTaskStateFragment' };

export type JobFragment = { __typename?: 'Job', id?: string | null, queueName: string, name: string, failedReason?: string | null, state: string, data?: (
    { __typename?: 'JobBackupData' }
    & { ' $fragmentRefs'?: { 'JobBackupDataFragment': JobBackupDataFragment } }
  ) | (
    { __typename?: 'JobCleanupData' }
    & { ' $fragmentRefs'?: { 'JobCleanupDataFragment': JobCleanupDataFragment } }
  ) | (
    { __typename?: 'JobFsckData' }
    & { ' $fragmentRefs'?: { 'JobFsckDataFragment': JobFsckDataFragment } }
  ) | (
    { __typename?: 'JobRemoveData' }
    & { ' $fragmentRefs'?: { 'JobRemoveDataFragment': JobRemoveDataFragment } }
  ) | (
    { __typename?: 'JobRestoreData' }
    & { ' $fragmentRefs'?: { 'JobRestoreDataFragment': JobRestoreDataFragment } }
  ) | null, progression?: (
    { __typename?: 'BackupTaskState' }
    & { ' $fragmentRefs'?: { 'BackupTaskStateFragment': BackupTaskStateFragment } }
  ) | (
    { __typename?: 'CleanerTaskState' }
    & { ' $fragmentRefs'?: { 'CleanerTaskStateFragment': CleanerTaskStateFragment } }
  ) | (
    { __typename?: 'FsckTaskState' }
    & { ' $fragmentRefs'?: { 'FsckTaskStateFragment': FsckTaskStateFragment } }
  ) | (
    { __typename?: 'RemoveTaskState' }
    & { ' $fragmentRefs'?: { 'RemoveTaskStateFragment': RemoveTaskStateFragment } }
  ) | (
    { __typename?: 'RestoreTaskState' }
    & { ' $fragmentRefs'?: { 'RestoreTaskStateFragment': RestoreTaskStateFragment } }
  ) | null } & { ' $fragmentName'?: 'JobFragment' };

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

export const EventBackupInformationFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EventBackupInformation"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventBackupInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"hostname"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"sharePath"}}]}}]} as unknown as DocumentNode<EventBackupInformationFragment, unknown>;
export const EventPoolInformationFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EventPoolInformation"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventPoolInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"fix"}},{"kind":"Field","name":{"kind":"Name","value":"refcount"}},{"kind":"Field","name":{"kind":"Name","value":"refcountError"}},{"kind":"Field","name":{"kind":"Name","value":"inUnused"}},{"kind":"Field","name":{"kind":"Name","value":"inRefcnt"}},{"kind":"Field","name":{"kind":"Name","value":"inNothing"}},{"kind":"Field","name":{"kind":"Name","value":"missing"}},{"kind":"Field","name":{"kind":"Name","value":"chunkCount"}},{"kind":"Field","name":{"kind":"Name","value":"chunkError"}}]}}]} as unknown as DocumentNode<EventPoolInformationFragment, unknown>;
export const EventPoolCleanedInformationFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EventPoolCleanedInformation"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventPoolCleanedInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"count"}}]}}]} as unknown as DocumentNode<EventPoolCleanedInformationFragment, unknown>;
export const EventHashConversionInformationFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EventHashConversionInformation"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventHashConversionInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"count"}},{"kind":"Field","name":{"kind":"Name","value":"algorithm"}}]}}]} as unknown as DocumentNode<EventHashConversionInformationFragment, unknown>;
export const ApplicationEventFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ApplicationEvent"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"ApplicationEvent"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"uuid"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"step"}},{"kind":"Field","name":{"kind":"Name","value":"source"}},{"kind":"Field","name":{"kind":"Name","value":"timestamp"}},{"kind":"Field","name":{"kind":"Name","value":"errorMessages"}},{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"information"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventBackupInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"EventBackupInformation"}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventPoolInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"EventPoolInformation"}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventPoolCleanedInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"EventPoolCleanedInformation"}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventHashConversionInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"EventHashConversionInformation"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EventBackupInformation"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventBackupInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"hostname"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"sharePath"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EventPoolInformation"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventPoolInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"fix"}},{"kind":"Field","name":{"kind":"Name","value":"refcount"}},{"kind":"Field","name":{"kind":"Name","value":"refcountError"}},{"kind":"Field","name":{"kind":"Name","value":"inUnused"}},{"kind":"Field","name":{"kind":"Name","value":"inRefcnt"}},{"kind":"Field","name":{"kind":"Name","value":"inNothing"}},{"kind":"Field","name":{"kind":"Name","value":"missing"}},{"kind":"Field","name":{"kind":"Name","value":"chunkCount"}},{"kind":"Field","name":{"kind":"Name","value":"chunkError"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EventPoolCleanedInformation"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventPoolCleanedInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"count"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EventHashConversionInformation"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventHashConversionInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"count"}},{"kind":"Field","name":{"kind":"Name","value":"algorithm"}}]}}]} as unknown as DocumentNode<ApplicationEventFragment, unknown>;
export const FragmentFileDescriptionFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"FragmentFileDescription"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"FileDescription"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"path"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"stats"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"ownerId"}},{"kind":"Field","name":{"kind":"Name","value":"groupId"}},{"kind":"Field","name":{"kind":"Name","value":"mode"}},{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"lastModified"}}]}},{"kind":"Field","name":{"kind":"Name","value":"symlink"}}]}}]} as unknown as DocumentNode<FragmentFileDescriptionFragment, unknown>;
export const JobPoolResponseFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobPoolResponse"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobResponse"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]} as unknown as DocumentNode<JobPoolResponseFragment, unknown>;
export const JobBackupDataFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobBackupData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobBackupData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"ip"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}}]}}]} as unknown as DocumentNode<JobBackupDataFragment, unknown>;
export const JobRestoreDataFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobRestoreData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobRestoreData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"ip"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"destinationDirectory"}},{"kind":"Field","name":{"kind":"Name","value":"files"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"share"}},{"kind":"Field","name":{"kind":"Name","value":"selection"}}]}}]}}]} as unknown as DocumentNode<JobRestoreDataFragment, unknown>;
export const JobRemoveDataFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobRemoveData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobRemoveData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}}]}}]} as unknown as DocumentNode<JobRemoveDataFragment, unknown>;
export const JobCleanupDataFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobCleanupData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobCleanupData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"target"}}]}}]} as unknown as DocumentNode<JobCleanupDataFragment, unknown>;
export const JobFsckDataFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobFsckData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobFsckData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"dryRun"}},{"kind":"Field","name":{"kind":"Name","value":"verifyChunks"}}]}}]} as unknown as DocumentNode<JobFsckDataFragment, unknown>;
export const BackupTaskStateFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BackupTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"BackupTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"backupExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"backupErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"backupErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","alias":{"kind":"Name","value":"backupProgress"},"name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"startTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"endTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}}]}},{"kind":"Field","name":{"kind":"Name","value":"preCommandStates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"executionState"}},{"kind":"Field","name":{"kind":"Name","value":"command"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"command"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"shareStates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"share"}},{"kind":"Field","name":{"kind":"Name","value":"executionState"}},{"kind":"Field","name":{"kind":"Name","value":"backupProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"startTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"endTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}}]}},{"kind":"Field","name":{"kind":"Name","value":"fileListProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"postCommandStates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"executionState"}},{"kind":"Field","name":{"kind":"Name","value":"command"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"command"}}]}}]}}]}}]} as unknown as DocumentNode<BackupTaskStateFragment, unknown>;
export const RestoreTaskStateFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RestoreTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"RestoreTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"restoreExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"restoreErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"restoreErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","alias":{"kind":"Name","value":"restoreProgression"},"name":{"kind":"Name","value":"globalProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"startTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"endTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}}]}}]}}]} as unknown as DocumentNode<RestoreTaskStateFragment, unknown>;
export const RemoveTaskStateFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RemoveTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"RemoveTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"removeExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"removeErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"removeErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}}]}}]} as unknown as DocumentNode<RemoveTaskStateFragment, unknown>;
export const CleanerTaskStateFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"CleanerTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"CleanerTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"cleanerExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"cleanerErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"cleanerErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","alias":{"kind":"Name","value":"cleanerProgress"},"name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}}]}}]}}]} as unknown as DocumentNode<CleanerTaskStateFragment, unknown>;
export const FsckTaskStateFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"FsckTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"FsckTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"fsckExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"fsckErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"fsckErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","name":{"kind":"Name","value":"dryRun"}},{"kind":"Field","name":{"kind":"Name","value":"refcntProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"totalCount"}}]}},{"kind":"Field","name":{"kind":"Name","value":"unusedProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"inNothing"}},{"kind":"Field","name":{"kind":"Name","value":"inRefcnt"}},{"kind":"Field","name":{"kind":"Name","value":"inUnused"}},{"kind":"Field","name":{"kind":"Name","value":"missing"}}]}},{"kind":"Field","name":{"kind":"Name","value":"chunkProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"totalCount"}}]}}]}}]} as unknown as DocumentNode<FsckTaskStateFragment, unknown>;
export const JobFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Job"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Job"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"queueName"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"failedReason"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobBackupData"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobRestoreData"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobRemoveData"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobCleanupData"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobFsckData"}}]}},{"kind":"Field","name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BackupTaskState"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"RestoreTaskState"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"RemoveTaskState"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"CleanerTaskState"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"FsckTaskState"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobBackupData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobBackupData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"ip"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobRestoreData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobRestoreData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"ip"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"destinationDirectory"}},{"kind":"Field","name":{"kind":"Name","value":"files"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"share"}},{"kind":"Field","name":{"kind":"Name","value":"selection"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobRemoveData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobRemoveData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobCleanupData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobCleanupData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"target"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobFsckData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobFsckData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"dryRun"}},{"kind":"Field","name":{"kind":"Name","value":"verifyChunks"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BackupTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"BackupTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"backupExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"backupErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"backupErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","alias":{"kind":"Name","value":"backupProgress"},"name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"startTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"endTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}}]}},{"kind":"Field","name":{"kind":"Name","value":"preCommandStates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"executionState"}},{"kind":"Field","name":{"kind":"Name","value":"command"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"command"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"shareStates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"share"}},{"kind":"Field","name":{"kind":"Name","value":"executionState"}},{"kind":"Field","name":{"kind":"Name","value":"backupProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"startTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"endTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}}]}},{"kind":"Field","name":{"kind":"Name","value":"fileListProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"postCommandStates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"executionState"}},{"kind":"Field","name":{"kind":"Name","value":"command"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"command"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RestoreTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"RestoreTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"restoreExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"restoreErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"restoreErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","alias":{"kind":"Name","value":"restoreProgression"},"name":{"kind":"Name","value":"globalProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"startTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"endTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RemoveTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"RemoveTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"removeExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"removeErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"removeErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"CleanerTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"CleanerTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"cleanerExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"cleanerErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"cleanerErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","alias":{"kind":"Name","value":"cleanerProgress"},"name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"FsckTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"FsckTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"fsckExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"fsckErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"fsckErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","name":{"kind":"Name","value":"dryRun"}},{"kind":"Field","name":{"kind":"Name","value":"refcntProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"totalCount"}}]}},{"kind":"Field","name":{"kind":"Name","value":"unusedProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"inNothing"}},{"kind":"Field","name":{"kind":"Name","value":"inRefcnt"}},{"kind":"Field","name":{"kind":"Name","value":"inUnused"}},{"kind":"Field","name":{"kind":"Name","value":"missing"}}]}},{"kind":"Field","name":{"kind":"Name","value":"chunkProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"totalCount"}}]}}]}}]} as unknown as DocumentNode<JobFragment, unknown>;
export const HostDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"Host"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"hostname"},"value":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"agentVersion"}},{"kind":"Field","name":{"kind":"Name","value":"lastBackup"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"agentVersion"}},{"kind":"Field","name":{"kind":"Name","value":"status"}}]}},{"kind":"Field","name":{"kind":"Name","value":"availibilityState"}},{"kind":"Field","name":{"kind":"Name","value":"timeSinceLastBackup"}},{"kind":"Field","name":{"kind":"Name","value":"dateToNextBackup"}},{"kind":"Field","name":{"kind":"Name","value":"addresses"}},{"kind":"Field","name":{"kind":"Name","value":"configuration"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"operations"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"preCommands"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"command"}}]}},{"kind":"Field","name":{"kind":"Name","value":"operation"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"shares"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"postCommands"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"command"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"schedule"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"activated"}}]}}]}}]}}]}}]} as unknown as DocumentNode<HostQuery, HostQueryVariables>;
export const HostsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"Hosts"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"hosts"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"lastBackup"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"agentVersion"}}]}},{"kind":"Field","name":{"kind":"Name","value":"agentVersion"}},{"kind":"Field","name":{"kind":"Name","value":"availibilityState"}},{"kind":"Field","name":{"kind":"Name","value":"timeSinceLastBackup"}},{"kind":"Field","name":{"kind":"Name","value":"dateToNextBackup"}},{"kind":"Field","name":{"kind":"Name","value":"configuration"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"schedule"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"activated"}}]}}]}}]}}]}}]} as unknown as DocumentNode<HostsQuery, HostsQueryVariables>;
export const BackupDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"Backup"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"number"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"backup"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"hostname"},"value":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}}},{"kind":"Argument","name":{"kind":"Name","value":"number"},"value":{"kind":"Variable","name":{"kind":"Name","value":"number"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"endDate"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"existingFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"existingFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}}]}}]}}]} as unknown as DocumentNode<BackupQuery, BackupQueryVariables>;
export const BackupsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"Backups"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"backups"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"hostname"},"value":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"endDate"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"existingFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"existingFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}}]}}]}}]} as unknown as DocumentNode<BackupsQuery, BackupsQueryVariables>;
export const BackupsBrowseDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"BackupsBrowse"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"number"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"sharePath"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"path"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"backup"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"hostname"},"value":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}}},{"kind":"Argument","name":{"kind":"Name","value":"number"},"value":{"kind":"Variable","name":{"kind":"Name","value":"number"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"files"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"sharePath"},"value":{"kind":"Variable","name":{"kind":"Name","value":"sharePath"}}},{"kind":"Argument","name":{"kind":"Name","value":"path"},"value":{"kind":"Variable","name":{"kind":"Name","value":"path"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"FragmentFileDescription"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"FragmentFileDescription"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"FileDescription"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"path"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"stats"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"ownerId"}},{"kind":"Field","name":{"kind":"Name","value":"groupId"}},{"kind":"Field","name":{"kind":"Name","value":"mode"}},{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"lastModified"}}]}},{"kind":"Field","name":{"kind":"Name","value":"symlink"}}]}}]} as unknown as DocumentNode<BackupsBrowseQuery, BackupsBrowseQueryVariables>;
export const CreateBackupDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"createBackup"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"createBackup"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"hostname"},"value":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]} as unknown as DocumentNode<CreateBackupMutation, CreateBackupMutationVariables>;
export const RemoveBackupDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"removeBackup"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"number"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"removeBackup"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"hostname"},"value":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}}},{"kind":"Argument","name":{"kind":"Name","value":"number"},"value":{"kind":"Variable","name":{"kind":"Name","value":"number"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]} as unknown as DocumentNode<RemoveBackupMutation, RemoveBackupMutationVariables>;
export const SharesBrowseDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"SharesBrowse"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"number"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"backup"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"hostname"},"value":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}}},{"kind":"Argument","name":{"kind":"Name","value":"number"},"value":{"kind":"Variable","name":{"kind":"Name","value":"number"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"shares"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"FragmentFileDescription"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"FragmentFileDescription"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"FileDescription"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"path"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"stats"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"ownerId"}},{"kind":"Field","name":{"kind":"Name","value":"groupId"}},{"kind":"Field","name":{"kind":"Name","value":"mode"}},{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"lastModified"}}]}},{"kind":"Field","name":{"kind":"Name","value":"symlink"}}]}}]} as unknown as DocumentNode<SharesBrowseQuery, SharesBrowseQueryVariables>;
export const RestoreBackupDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"restoreBackup"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"RestoreInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"restoreBackup"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobPoolResponse"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobPoolResponse"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobResponse"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]} as unknown as DocumentNode<RestoreBackupMutation, RestoreBackupMutationVariables>;
export const ClearCacheDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"clearCache"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"clearCache"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"void"}}]}}]}}]} as unknown as DocumentNode<ClearCacheMutation, ClearCacheMutationVariables>;
export const EventsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"Events"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"firstEvent"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"DateTime"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"lastEvent"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"DateTime"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"events"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"firstEvent"},"value":{"kind":"Variable","name":{"kind":"Name","value":"firstEvent"}}},{"kind":"Argument","name":{"kind":"Name","value":"lastEvent"},"value":{"kind":"Variable","name":{"kind":"Name","value":"lastEvent"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ApplicationEvent"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EventBackupInformation"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventBackupInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"hostname"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"sharePath"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EventPoolInformation"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventPoolInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"fix"}},{"kind":"Field","name":{"kind":"Name","value":"refcount"}},{"kind":"Field","name":{"kind":"Name","value":"refcountError"}},{"kind":"Field","name":{"kind":"Name","value":"inUnused"}},{"kind":"Field","name":{"kind":"Name","value":"inRefcnt"}},{"kind":"Field","name":{"kind":"Name","value":"inNothing"}},{"kind":"Field","name":{"kind":"Name","value":"missing"}},{"kind":"Field","name":{"kind":"Name","value":"chunkCount"}},{"kind":"Field","name":{"kind":"Name","value":"chunkError"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EventPoolCleanedInformation"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventPoolCleanedInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"count"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EventHashConversionInformation"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventHashConversionInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"count"}},{"kind":"Field","name":{"kind":"Name","value":"algorithm"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ApplicationEvent"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"ApplicationEvent"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"uuid"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"step"}},{"kind":"Field","name":{"kind":"Name","value":"source"}},{"kind":"Field","name":{"kind":"Name","value":"timestamp"}},{"kind":"Field","name":{"kind":"Name","value":"errorMessages"}},{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"information"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventBackupInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"EventBackupInformation"}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventPoolInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"EventPoolInformation"}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventPoolCleanedInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"EventPoolCleanedInformation"}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventHashConversionInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"EventHashConversionInformation"}}]}}]}}]}}]} as unknown as DocumentNode<EventsQuery, EventsQueryVariables>;
export const CleanupPoolDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"cleanupPool"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"cleanupPool"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobPoolResponse"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobPoolResponse"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobResponse"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]} as unknown as DocumentNode<CleanupPoolMutation, CleanupPoolMutationVariables>;
export const CheckAndFixPoolDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"checkAndFixPool"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"fix"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Boolean"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"verifyChunks"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Boolean"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"checkAndFixPool"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"fix"},"value":{"kind":"Variable","name":{"kind":"Name","value":"fix"}}},{"kind":"Argument","name":{"kind":"Name","value":"verifyChunks"},"value":{"kind":"Variable","name":{"kind":"Name","value":"verifyChunks"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobPoolResponse"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobPoolResponse"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobResponse"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]} as unknown as DocumentNode<CheckAndFixPoolMutation, CheckAndFixPoolMutationVariables>;
export const ServerInformationsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"ServerInformations"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"informations"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"platform"}},{"kind":"Field","name":{"kind":"Name","value":"uptime"}},{"kind":"Field","name":{"kind":"Name","value":"hostname"}},{"kind":"Field","name":{"kind":"Name","value":"woodstockVersion"}}]}}]}}]} as unknown as DocumentNode<ServerInformationsQuery, ServerInformationsQueryVariables>;
export const DiskUsageStatisticsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"DiskUsageStatistics"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"statistics"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"hosts"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"compressedSize"}}]}}]}}]}}]} as unknown as DocumentNode<DiskUsageStatisticsQuery, DiskUsageStatisticsQueryVariables>;
export const QueueStatisticsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"QueueStatistics"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"queueStats"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"active"}},{"kind":"Field","name":{"kind":"Name","value":"waiting"}},{"kind":"Field","name":{"kind":"Name","value":"failed"}},{"kind":"Field","name":{"kind":"Name","value":"delayed"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}}]}}]}}]} as unknown as DocumentNode<QueueStatisticsQuery, QueueStatisticsQueryVariables>;
export const PoolStatisticsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"PoolStatistics"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"statistics"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"diskUsage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"used"}},{"kind":"Field","name":{"kind":"Name","value":"usedLastMonth"}},{"kind":"Field","name":{"kind":"Name","value":"free"}},{"kind":"Field","name":{"kind":"Name","value":"total"}}]}},{"kind":"Field","name":{"kind":"Name","value":"poolUsage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"nbChunk"}},{"kind":"Field","name":{"kind":"Name","value":"nbChunkLastMonth"}},{"kind":"Field","name":{"kind":"Name","value":"nbChunkRange"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"time"}},{"kind":"Field","name":{"kind":"Name","value":"value"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nbRef"}},{"kind":"Field","name":{"kind":"Name","value":"nbRefLastMonth"}},{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"compressedSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedSizeLastMonth"}},{"kind":"Field","name":{"kind":"Name","value":"compressedSizeRange"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"time"}},{"kind":"Field","name":{"kind":"Name","value":"value"}}]}},{"kind":"Field","name":{"kind":"Name","value":"unusedSize"}}]}}]}}]}}]} as unknown as DocumentNode<PoolStatisticsQuery, PoolStatisticsQueryVariables>;
export const TasksDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"Tasks"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"QueueListInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"queue"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Job"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobBackupData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobBackupData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"ip"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobRestoreData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobRestoreData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"ip"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"destinationDirectory"}},{"kind":"Field","name":{"kind":"Name","value":"files"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"share"}},{"kind":"Field","name":{"kind":"Name","value":"selection"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobRemoveData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobRemoveData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobCleanupData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobCleanupData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"target"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobFsckData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobFsckData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"dryRun"}},{"kind":"Field","name":{"kind":"Name","value":"verifyChunks"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BackupTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"BackupTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"backupExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"backupErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"backupErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","alias":{"kind":"Name","value":"backupProgress"},"name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"startTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"endTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}}]}},{"kind":"Field","name":{"kind":"Name","value":"preCommandStates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"executionState"}},{"kind":"Field","name":{"kind":"Name","value":"command"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"command"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"shareStates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"share"}},{"kind":"Field","name":{"kind":"Name","value":"executionState"}},{"kind":"Field","name":{"kind":"Name","value":"backupProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"startTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"endTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}}]}},{"kind":"Field","name":{"kind":"Name","value":"fileListProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"postCommandStates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"executionState"}},{"kind":"Field","name":{"kind":"Name","value":"command"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"command"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RestoreTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"RestoreTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"restoreExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"restoreErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"restoreErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","alias":{"kind":"Name","value":"restoreProgression"},"name":{"kind":"Name","value":"globalProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"startTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"endTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RemoveTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"RemoveTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"removeExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"removeErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"removeErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"CleanerTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"CleanerTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"cleanerExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"cleanerErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"cleanerErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","alias":{"kind":"Name","value":"cleanerProgress"},"name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"FsckTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"FsckTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"fsckExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"fsckErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"fsckErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","name":{"kind":"Name","value":"dryRun"}},{"kind":"Field","name":{"kind":"Name","value":"refcntProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"totalCount"}}]}},{"kind":"Field","name":{"kind":"Name","value":"unusedProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"inNothing"}},{"kind":"Field","name":{"kind":"Name","value":"inRefcnt"}},{"kind":"Field","name":{"kind":"Name","value":"inUnused"}},{"kind":"Field","name":{"kind":"Name","value":"missing"}}]}},{"kind":"Field","name":{"kind":"Name","value":"chunkProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"totalCount"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Job"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Job"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"queueName"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"failedReason"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobBackupData"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobRestoreData"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobRemoveData"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobCleanupData"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobFsckData"}}]}},{"kind":"Field","name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BackupTaskState"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"RestoreTaskState"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"RemoveTaskState"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"CleanerTaskState"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"FsckTaskState"}}]}}]}}]} as unknown as DocumentNode<TasksQuery, TasksQueryVariables>;
export const QueueTasksJobUpdatedDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"subscription","name":{"kind":"Name","value":"QueueTasksJobUpdated"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"jobUpdated"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Job"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobBackupData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobBackupData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"ip"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobRestoreData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobRestoreData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"ip"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"destinationDirectory"}},{"kind":"Field","name":{"kind":"Name","value":"files"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"share"}},{"kind":"Field","name":{"kind":"Name","value":"selection"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobRemoveData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobRemoveData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobCleanupData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobCleanupData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"target"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobFsckData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobFsckData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"dryRun"}},{"kind":"Field","name":{"kind":"Name","value":"verifyChunks"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BackupTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"BackupTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"backupExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"backupErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"backupErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","alias":{"kind":"Name","value":"backupProgress"},"name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"startTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"endTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}}]}},{"kind":"Field","name":{"kind":"Name","value":"preCommandStates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"executionState"}},{"kind":"Field","name":{"kind":"Name","value":"command"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"command"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"shareStates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"share"}},{"kind":"Field","name":{"kind":"Name","value":"executionState"}},{"kind":"Field","name":{"kind":"Name","value":"backupProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"startTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"endTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}}]}},{"kind":"Field","name":{"kind":"Name","value":"fileListProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"postCommandStates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"executionState"}},{"kind":"Field","name":{"kind":"Name","value":"command"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"command"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RestoreTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"RestoreTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"restoreExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"restoreErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"restoreErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","alias":{"kind":"Name","value":"restoreProgression"},"name":{"kind":"Name","value":"globalProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"startTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"endTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RemoveTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"RemoveTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"removeExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"removeErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"removeErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"CleanerTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"CleanerTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"cleanerExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"cleanerErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"cleanerErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","alias":{"kind":"Name","value":"cleanerProgress"},"name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"FsckTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"FsckTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"fsckExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"fsckErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"fsckErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","name":{"kind":"Name","value":"dryRun"}},{"kind":"Field","name":{"kind":"Name","value":"refcntProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"totalCount"}}]}},{"kind":"Field","name":{"kind":"Name","value":"unusedProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"inNothing"}},{"kind":"Field","name":{"kind":"Name","value":"inRefcnt"}},{"kind":"Field","name":{"kind":"Name","value":"inUnused"}},{"kind":"Field","name":{"kind":"Name","value":"missing"}}]}},{"kind":"Field","name":{"kind":"Name","value":"chunkProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"totalCount"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Job"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Job"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"queueName"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"failedReason"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobBackupData"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobRestoreData"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobRemoveData"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobCleanupData"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobFsckData"}}]}},{"kind":"Field","name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BackupTaskState"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"RestoreTaskState"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"RemoveTaskState"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"CleanerTaskState"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"FsckTaskState"}}]}}]}}]} as unknown as DocumentNode<QueueTasksJobUpdatedSubscription, QueueTasksJobUpdatedSubscriptionVariables>;
import { bigintTypePolicy } from '../utils/bigint.utils';

export const scalarTypePolicies = {
  Backup: {
    fields: {
      compressedFileSize: bigintTypePolicy,
      existingCompressedFileSize: bigintTypePolicy,
      existingFileSize: bigintTypePolicy,
      fileSize: bigintTypePolicy,
      modifiedCompressedFileSize: bigintTypePolicy,
      modifiedFileSize: bigintTypePolicy,
      newCompressedFileSize: bigintTypePolicy,
      newFileSize: bigintTypePolicy,
    },
  },
  BackupProgression: {
    fields: {
      compressedFileSize: bigintTypePolicy,
      fileSize: bigintTypePolicy,
      modifiedCompressedFileSize: bigintTypePolicy,
      modifiedFileSize: bigintTypePolicy,
      newCompressedFileSize: bigintTypePolicy,
      newFileSize: bigintTypePolicy,
      progressCurrent: bigintTypePolicy,
      progressMax: bigintTypePolicy,
    },
  },
  BigIntTimeSerie: { fields: { value: bigintTypePolicy } },
  CleanerProgression: { fields: { compressedFileSize: bigintTypePolicy, fileSize: bigintTypePolicy } },
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
  EventPoolCleanedInformation: { fields: { size: bigintTypePolicy } },
  FileListProgression: {
    fields: { fileSize: bigintTypePolicy, modifiedFileSize: bigintTypePolicy, newFileSize: bigintTypePolicy },
  },
  FileStat: {
    fields: { dev: bigintTypePolicy, ino: bigintTypePolicy, nlink: bigintTypePolicy, rdev: bigintTypePolicy },
  },
  HostStatistics: {
    fields: {
      compressedSize: bigintTypePolicy,
      compressedSizeLastMonth: bigintTypePolicy,
      size: bigintTypePolicy,
      sizeLastMonth: bigintTypePolicy,
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
