/* eslint-disable */
import type { TypedDocumentNode as DocumentNode } from '@graphql-typed-document-node/core';
export type Maybe<T> = T | null;
export type InputMaybe<T> = T | null | undefined;
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
  BigInt: { input: bigint; output: bigint; }
  Buffer: { input: string; output: string; }
  /**
   * Implement the DateTime<Local> scalar
   *
   * The input/output is a string in RFC3339 format.
   */
  DateTime: { input: Date; output: Date; }
  /** A scalar that can represent any JSON Object value. */
  JSONObject: { input: any; output: any; }
};

export enum AbortingStageDto {
  ToAddInPool = 'TO_ADD_IN_POOL',
  ToCompact = 'TO_COMPACT',
  ToCountRef = 'TO_COUNT_REF'
}

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

export enum ArchiveFormat {
  Dir = 'DIR',
  Tar = 'TAR',
  TarGz = 'TAR_GZ',
  TarXz = 'TAR_XZ',
  TarZstd = 'TAR_ZSTD'
}

export enum ArchiveHostExecutionState {
  Failed = 'FAILED',
  InProgress = 'IN_PROGRESS',
  Success = 'SUCCESS',
  Waiting = 'WAITING'
}

export type ArchiveHostState = {
  __typename?: 'ArchiveHostState';
  /**
   * `None` until this host reaches `Success`/`Failed` — see the format
   * mapping documented on `woodstock::archiving::ArchiveHostState::archive_size`.
   */
  archiveSize?: Maybe<Scalars['BigInt']['output']>;
  executionState: ArchiveHostExecutionState;
  fileCount: Scalars['Int']['output'];
  hostname: Scalars['String']['output'];
  percent: Scalars['Float']['output'];
  progressCurrent: Scalars['BigInt']['output'];
  progressMax: Scalars['BigInt']['output'];
};

/**
 * Read-only view of an archive profile from `archiving.yml` — there is no
 * GraphQL mutation to create/edit profiles, only to list them (for the
 * manual-trigger UI) and to trigger a run.
 */
export type ArchiveProfile = {
  __typename?: 'ArchiveProfile';
  checksum: Scalars['Boolean']['output'];
  /** Tar-family only; `None` means "use the codec's own recommended default". */
  compressionLevel?: Maybe<Scalars['Int']['output']>;
  destination: Scalars['String']['output'];
  enabled: Scalars['Boolean']['output'];
  format: ArchiveFormat;
  /** Populated only when `hostSelectionMode` is `INCLUDE` or `EXCLUDE`. */
  hostSelectionHosts?: Maybe<Array<Scalars['String']['output']>>;
  hostSelectionMode: HostSelectionMode;
  /** Populated only when `hostSelectionMode` is `GLOB`. */
  hostSelectionPattern?: Maybe<Scalars['String']['output']>;
  name: Scalars['String']['output'];
  scheduleCron: Scalars['String']['output'];
};

export type ArchiveRunResponse = {
  __typename?: 'ArchiveRunResponse';
  jobIds: Array<Scalars['String']['output']>;
};

export enum BackupErrorState {
  AddReferencesToPoolError = 'ADD_REFERENCES_TO_POOL_ERROR',
  AuthenticationError = 'AUTHENTICATION_ERROR',
  BackupError = 'BACKUP_ERROR',
  CommandExecutionError = 'COMMAND_EXECUTION_ERROR',
  CompactError = 'COMPACT_ERROR',
  CountReferencesError = 'COUNT_REFERENCES_ERROR',
  InitializationError = 'INITIALIZATION_ERROR',
  Unknown = 'UNKNOWN'
}

export type BackupEx = {
  __typename?: 'BackupEx';
  agentVersion?: Maybe<Scalars['String']['output']>;
  compressedFileSize: Scalars['BigInt']['output'];
  endDate?: Maybe<Scalars['DateTime']['output']>;
  errorCount: Scalars['Int']['output'];
  errorMessage?: Maybe<Scalars['String']['output']>;
  existingCompressedFileSize: Scalars['BigInt']['output'];
  existingFileCount: Scalars['Int']['output'];
  existingFileSize: Scalars['BigInt']['output'];
  fileCount: Scalars['Int']['output'];
  fileSize: Scalars['BigInt']['output'];
  files: Array<FileDescription>;
  id: Scalars['ID']['output'];
  modifiedCompressedFileSize: Scalars['BigInt']['output'];
  modifiedFileCount: Scalars['Int']['output'];
  modifiedFileSize: Scalars['BigInt']['output'];
  newCompressedFileSize: Scalars['BigInt']['output'];
  newFileCount: Scalars['Int']['output'];
  newFileSize: Scalars['BigInt']['output'];
  number: Scalars['Int']['output'];
  removedFileCount: Scalars['Int']['output'];
  retentionCategory?: Maybe<RetentionCategoryDto>;
  shareRecords: Array<BackupShareRecord>;
  shares: Array<FileDescription>;
  speed: Scalars['Float']['output'];
  startDate: Scalars['DateTime']['output'];
  status: BackupStatusDto;
};


export type BackupExFilesArgs = {
  path: Scalars['Buffer']['input'];
  sharePath: Scalars['String']['input'];
};

export enum BackupExecutionState {
  AddReferencesToPool = 'ADD_REFERENCES_TO_POOL',
  Authenticate = 'AUTHENTICATE',
  Compact = 'COMPACT',
  Completed = 'COMPLETED',
  CountReferences = 'COUNT_REFERENCES',
  DownloadChunks = 'DOWNLOAD_CHUNKS',
  DownloadFileList = 'DOWNLOAD_FILE_LIST',
  Initialization = 'INITIALIZATION',
  PostCommands = 'POST_COMMANDS',
  PreCommands = 'PRE_COMMANDS',
  Skipped = 'SKIPPED',
  Waiting = 'WAITING'
}

export type BackupOperation = {
  __typename?: 'BackupOperation';
  excludes?: Maybe<Array<Scalars['String']['output']>>;
  includes?: Maybe<Array<Scalars['String']['output']>>;
  shares: Array<BackupTaskShare>;
  timeout?: Maybe<Scalars['Int']['output']>;
};

export type BackupProgression = {
  __typename?: 'BackupProgression';
  compressedFileSize: Scalars['BigInt']['output'];
  endTransferDate?: Maybe<Scalars['DateTime']['output']>;
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
  startDate: Scalars['DateTime']['output'];
  startTransferDate?: Maybe<Scalars['DateTime']['output']>;
};

export type BackupQueueData = JobArchiveData | JobBackupData | JobCleanupData | JobFsckData | JobRemoveData | JobRestoreData | JobStatsData;

export type BackupQueueProgress = JobArchiveTaskState | JobBackupTaskState | JobCleanerTaskState | JobFsckTaskState | JobRemoveState | JobRestoreTaskState;

/** A share record for a completed backup — path + snapshot method used. */
export type BackupShareRecord = {
  __typename?: 'BackupShareRecord';
  path: Scalars['String']['output'];
  snapshotFailureReason?: Maybe<Scalars['String']['output']>;
  snapshotMethod: SnapshotMethodDto;
};

export type BackupStatusDto = {
  __typename?: 'BackupStatusDto';
  abortingStage?: Maybe<AbortingStageDto>;
  failedStage?: Maybe<FailedStageDto>;
  finishingStage?: Maybe<FinishingStageDto>;
  removingStage?: Maybe<RemovingStageDto>;
  statusType: BackupStatusTypeDto;
};

export enum BackupStatusTypeDto {
  Aborted = 'ABORTED',
  Aborting = 'ABORTING',
  Completed = 'COMPLETED',
  Failed = 'FAILED',
  Finishing = 'FINISHING',
  InProgress = 'IN_PROGRESS',
  Removing = 'REMOVING'
}

export type BackupTaskShare = {
  __typename?: 'BackupTaskShare';
  excludes?: Maybe<Array<Scalars['String']['output']>>;
  includes?: Maybe<Array<Scalars['String']['output']>>;
  name: Scalars['String']['output'];
};

export type BigIntTimeSerie = {
  __typename?: 'BigIntTimeSerie';
  time: Scalars['DateTime']['output'];
  value: Scalars['BigInt']['output'];
};

export enum ChunkAlgorithm {
  Blake_3 = 'BLAKE_3',
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
  ApplyingRefcntError = 'APPLYING_REFCNT_ERROR',
  CleaningError = 'CLEANING_ERROR',
  InitializationError = 'INITIALIZATION_ERROR',
  Unknown = 'UNKNOWN'
}

export enum CleanerExecutionState {
  ApplyingRefcnt = 'APPLYING_REFCNT',
  Cleaning = 'CLEANING',
  Completed = 'COMPLETED',
  Initialization = 'INITIALIZATION',
  Waiting = 'WAITING'
}

export type CleanerProgression = {
  __typename?: 'CleanerProgression';
  compressedFileSize: Scalars['BigInt']['output'];
  fileSize: Scalars['BigInt']['output'];
  progressCurrent: Scalars['Int']['output'];
  progressMax: Scalars['Int']['output'];
};

export type DiskUsage = {
  __typename?: 'DiskUsage';
  free: Scalars['BigInt']['output'];
  freeLastMonth: Scalars['BigInt']['output'];
  freeRange: Array<BigIntTimeSerie>;
  total: Scalars['BigInt']['output'];
  totalLastMonth: Scalars['BigInt']['output'];
  totalRange: Array<BigIntTimeSerie>;
  used: Scalars['BigInt']['output'];
  usedLastMonth: Scalars['BigInt']['output'];
  usedRange: Array<BigIntTimeSerie>;
};

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
  Cli = 'CLI',
  Import = 'IMPORT',
  User = 'USER',
  Woodstock = 'WOODSTOCK'
}

export enum EventStatus {
  ClientDisconnected = 'CLIENT_DISCONNECTED',
  GenericError = 'GENERIC_ERROR',
  None = 'NONE',
  ServerCrashed = 'SERVER_CRASHED',
  Success = 'SUCCESS'
}

export enum EventStep {
  End = 'END',
  Start = 'START'
}

export enum EventType {
  Backup = 'BACKUP',
  Delete = 'DELETE',
  HashConversion = 'HASH_CONVERSION',
  PoolChecked = 'POOL_CHECKED',
  PoolCleaned = 'POOL_CLEANED',
  Restore = 'RESTORE'
}

export enum ExecuteCommandExecutionState {
  Failed = 'FAILED',
  InProgress = 'IN_PROGRESS',
  Success = 'SUCCESS',
  Waiting = 'WAITING'
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

export enum FailedStageDto {
  Compact = 'COMPACT',
  InPool = 'IN_POOL',
  RefCount = 'REF_COUNT'
}

export type FileAcl = {
  __typename?: 'FileAcl';
  id: Scalars['Int']['output'];
  perm: Scalars['Int']['output'];
  qualifier: FileManifestAclQualifierDto;
};

export type FileDescription = {
  __typename?: 'FileDescription';
  acl: Array<FileAcl>;
  chunks: Array<Scalars['Buffer']['output']>;
  hash: Scalars['Buffer']['output'];
  metadata: Scalars['JSONObject']['output'];
  path: Scalars['Buffer']['output'];
  stats?: Maybe<FileStat>;
  symlink: Scalars['Buffer']['output'];
  type: FileManifestTypeDto;
  xattr: Array<FileXAttr>;
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

export enum FileManifestAclQualifierDto {
  GroupId = 'GROUP_ID',
  GroupObj = 'GROUP_OBJ',
  Mask = 'MASK',
  Other = 'OTHER',
  Undefined = 'UNDEFINED',
  UserId = 'USER_ID',
  UserObj = 'USER_OBJ'
}

export enum FileManifestTypeDto {
  BlockDevice = 'BLOCK_DEVICE',
  CharacterDevice = 'CHARACTER_DEVICE',
  Directory = 'DIRECTORY',
  Fifo = 'FIFO',
  RegularFile = 'REGULAR_FILE',
  Socket = 'SOCKET',
  Symlink = 'SYMLINK',
  Unknown = 'UNKNOWN'
}

export type FileStat = {
  __typename?: 'FileStat';
  compressedSize: Scalars['BigInt']['output'];
  created: Scalars['Int']['output'];
  dev: Scalars['BigInt']['output'];
  groupId: Scalars['Int']['output'];
  ino: Scalars['BigInt']['output'];
  lastModified: Scalars['Int']['output'];
  lastRead: Scalars['Int']['output'];
  mode: Scalars['Int']['output'];
  nlink: Scalars['BigInt']['output'];
  ownerId: Scalars['Int']['output'];
  rdev: Scalars['BigInt']['output'];
  size: Scalars['BigInt']['output'];
  type: FileManifestTypeDto;
};

export type FileXAttr = {
  __typename?: 'FileXAttr';
  key: Scalars['Buffer']['output'];
  value: Scalars['Buffer']['output'];
};

export enum FinishingStageDto {
  ToAddInPool = 'TO_ADD_IN_POOL',
  ToCompact = 'TO_COMPACT',
  ToCountRef = 'TO_COUNT_REF'
}

export enum FsckErrorState {
  ApplyingRefcntError = 'APPLYING_REFCNT_ERROR',
  InitializationError = 'INITIALIZATION_ERROR',
  Unknown = 'UNKNOWN',
  VerifyChunkError = 'VERIFY_CHUNK_ERROR',
  VerifyRefcntError = 'VERIFY_REFCNT_ERROR',
  VerifyUnusedError = 'VERIFY_UNUSED_ERROR'
}

export enum FsckExecutionState {
  ApplyingRefcnt = 'APPLYING_REFCNT',
  Completed = 'COMPLETED',
  Initialization = 'INITIALIZATION',
  VerifyChunk = 'VERIFY_CHUNK',
  VerifyRefcnt = 'VERIFY_REFCNT',
  VerifyUnused = 'VERIFY_UNUSED',
  Waiting = 'WAITING'
}

export type Host = {
  __typename?: 'Host';
  addresses?: Maybe<Array<Scalars['String']['output']>>;
  agentVersion?: Maybe<Scalars['String']['output']>;
  availibilityState?: Maybe<HostAvailibilityState>;
  backups: Array<BackupEx>;
  configuration: HostConfiguration;
  dateToNextBackup?: Maybe<Scalars['DateTime']['output']>;
  lastBackup?: Maybe<BackupEx>;
  name: Scalars['ID']['output'];
  timeSinceLastBackup?: Maybe<Scalars['Float']['output']>;
  timeToNextBackup?: Maybe<Scalars['Float']['output']>;
};

export enum HostAvailibilityState {
  Offline = 'OFFLINE',
  Online = 'ONLINE',
  Unknown = 'UNKNOWN'
}

export type HostConfigOperation = {
  __typename?: 'HostConfigOperation';
  operation?: Maybe<BackupOperation>;
  postCommands?: Maybe<Array<ExecuteCommandOperation>>;
  preCommands?: Maybe<Array<ExecuteCommandOperation>>;
};

export type HostConfiguration = {
  __typename?: 'HostConfiguration';
  addresses?: Maybe<Array<Scalars['String']['output']>>;
  operations: HostConfigOperation;
  port: Scalars['Int']['output'];
  schedule?: Maybe<Schedule>;
};

/**
 * The `mode` tag of an archive profile's `hostSelection` — which of the
 * mutually-exclusive `Glob`/`Include`/`Exclude` detail fields on
 * [`ArchiveProfile`] is populated.
 */
export enum HostSelectionMode {
  All = 'ALL',
  Exclude = 'EXCLUDE',
  Glob = 'GLOB',
  Include = 'INCLUDE'
}

export type HostStatistics = {
  __typename?: 'HostStatistics';
  compressedSize: Scalars['BigInt']['output'];
  compressedSizeLastMonth: Scalars['BigInt']['output'];
  compressedSizeRange: Array<BigIntTimeSerie>;
  host: Scalars['String']['output'];
  longestChain: Scalars['Int']['output'];
  longestChainLastMonth?: Maybe<Scalars['Int']['output']>;
  longestChainRange: Array<NumberTimeSerie>;
  nbChunk: Scalars['Int']['output'];
  nbChunkLastMonth?: Maybe<Scalars['Int']['output']>;
  nbChunkRange: Array<NumberTimeSerie>;
  nbRef: Scalars['Int']['output'];
  nbRefLastMonth?: Maybe<Scalars['Int']['output']>;
  nbRefRange: Array<NumberTimeSerie>;
  size: Scalars['BigInt']['output'];
  sizeLastMonth: Scalars['BigInt']['output'];
  sizeRange: Array<BigIntTimeSerie>;
};

export type Job = {
  __typename?: 'Job';
  data: BackupQueueData;
  failedReason?: Maybe<Scalars['String']['output']>;
  host?: Maybe<Scalars['String']['output']>;
  jobId: Scalars['String']['output'];
  kind: JobKind;
  progress?: Maybe<BackupQueueProgress>;
  status: JobStatus;
  timestamp: Scalars['Int']['output'];
};

export type JobArchiveData = {
  __typename?: 'JobArchiveData';
  hostnames: Array<Scalars['String']['output']>;
  profileName: Scalars['String']['output'];
};

export type JobArchiveTaskState = {
  __typename?: 'JobArchiveTaskState';
  archiveSize: Scalars['BigInt']['output'];
  /** Host currently being archived, `None` before the first host starts. */
  currentHost?: Maybe<Scalars['String']['output']>;
  failedHosts: Array<Scalars['String']['output']>;
  fileCount: Scalars['Int']['output'];
  hostStates: Array<ArchiveHostState>;
  hostsDone: Scalars['Int']['output'];
  hostsTotal: Scalars['Int']['output'];
  percent: Scalars['Float']['output'];
  progressCurrent: Scalars['BigInt']['output'];
  progressMax: Scalars['BigInt']['output'];
  speed: Scalars['Float']['output'];
};

export type JobBackupData = {
  __typename?: 'JobBackupData';
  config: HostConfiguration;
  force: Scalars['Boolean']['output'];
  host: Scalars['String']['output'];
  /** UUID v7 of this backup (primary key) */
  id: Scalars['String']['output'];
  ip?: Maybe<Scalars['String']['output']>;
  /** Sequential display number */
  number: Scalars['Int']['output'];
  previousId?: Maybe<Scalars['String']['output']>;
  startDate?: Maybe<Scalars['DateTime']['output']>;
};

export type JobBackupTaskState = {
  __typename?: 'JobBackupTaskState';
  errorMessage?: Maybe<Scalars['String']['output']>;
  errorState?: Maybe<BackupErrorState>;
  executionState: BackupExecutionState;
  postCommandStates: Array<ExecuteCommandState>;
  preCommandStates: Array<ExecuteCommandState>;
  progression: BackupProgression;
  shareStates: Array<ShareState>;
};

export type JobCleanerTaskState = {
  __typename?: 'JobCleanerTaskState';
  errorMessage?: Maybe<Scalars['String']['output']>;
  errorState?: Maybe<CleanerErrorState>;
  executionState: CleanerExecutionState;
  progression: CleanerProgression;
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

export type JobFsckTaskState = {
  __typename?: 'JobFsckTaskState';
  chunkProgression: ChunkProgression;
  dryRun: Scalars['Boolean']['output'];
  errorMessage?: Maybe<Scalars['String']['output']>;
  errorState?: Maybe<FsckErrorState>;
  executionState: FsckExecutionState;
  refcntProgression: RefcntProgression;
  unusedProgression: UnusedProgression;
};

export enum JobKind {
  Archive = 'ARCHIVE',
  Backup = 'BACKUP',
  CleanupRefcnt = 'CLEANUP_REFCNT',
  Fsck = 'FSCK',
  Remove = 'REMOVE',
  Restore = 'RESTORE',
  Stats = 'STATS'
}

export type JobRemoveData = {
  __typename?: 'JobRemoveData';
  config?: Maybe<HostConfiguration>;
  host: Scalars['String']['output'];
  /** UUID v7 of the backup to remove */
  id: Scalars['String']['output'];
  /** Sequential display number */
  number: Scalars['Int']['output'];
  startDate?: Maybe<Scalars['DateTime']['output']>;
};

export type JobRemoveState = {
  __typename?: 'JobRemoveState';
  errorMessage?: Maybe<Scalars['String']['output']>;
  errorState?: Maybe<RemoveErrorState>;
  executionState: RemoveExecutionState;
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
  /** UUID v7 of the backup to restore */
  id: Scalars['String']['output'];
  ip?: Maybe<Scalars['String']['output']>;
  /** Sequential display number */
  number: Scalars['Int']['output'];
  startDate?: Maybe<Scalars['DateTime']['output']>;
};

export type JobRestoreDataSelection = {
  __typename?: 'JobRestoreDataSelection';
  selection: Array<Scalars['String']['output']>;
  share: Scalars['String']['output'];
};

export type JobRestoreTaskState = {
  __typename?: 'JobRestoreTaskState';
  errorMessage?: Maybe<Scalars['String']['output']>;
  errorState?: Maybe<RestoreErrorState>;
  executionState: RestoreExecutionState;
  globalProgression: BackupProgression;
};

export type JobStatsData = {
  __typename?: 'JobStatsData';
  empty: Scalars['Boolean']['output'];
};

export enum JobStatus {
  Completed = 'COMPLETED',
  Created = 'CREATED',
  Failed = 'FAILED',
  Started = 'STARTED'
}

export type MutationRoot = {
  __typename?: 'MutationRoot';
  checkAndFixPool: JobResponse;
  cleanupPool: JobResponse;
  clearCache: JobResponse;
  createBackup: JobResponse;
  /**
   * Déclenche manuellement la purge des sauvegardes en surplus pour un hôte.
   *
   * Calcule immédiatement les sauvegardes à supprimer selon la politique de rétention
   * configurée pour l'hôte, puis enqueue un job `Remove` pour chacune.
   */
  purgeRetention: JobResponse;
  removeBackup: JobResponse;
  restoreBackup: JobResponse;
  /**
   * Triggers an archive profile run now — regardless of whether it is
   * enabled or due (same semantics as `ws_console archive run`). If
   * `host` is omitted, resolves the profile's full host selection and
   * enqueues one job per selected host.
   */
  runArchive: ArchiveRunResponse;
};


export type MutationRootCheckAndFixPoolArgs = {
  fix: Scalars['Boolean']['input'];
  verifyChunks: Scalars['Boolean']['input'];
};


export type MutationRootCreateBackupArgs = {
  hostname: Scalars['String']['input'];
};


export type MutationRootPurgeRetentionArgs = {
  hostname: Scalars['String']['input'];
};


export type MutationRootRemoveBackupArgs = {
  hostname: Scalars['String']['input'];
  id: Scalars['String']['input'];
};


export type MutationRootRestoreBackupArgs = {
  input: RestoreInput;
};


export type MutationRootRunArchiveArgs = {
  host?: InputMaybe<Scalars['String']['input']>;
  profile: Scalars['String']['input'];
};

export type NumberTimeSerie = {
  __typename?: 'NumberTimeSerie';
  time: Scalars['DateTime']['output'];
  value: Scalars['Int']['output'];
};

/** DTO for the overall health status of the storage pool. */
export type PoolHealthStatusDto = {
  __typename?: 'PoolHealthStatusDto';
  /** Overall health indicator (false if dirty state detected) */
  healthy: Scalars['Boolean']['output'];
  /** Whether the pool is in a dirty state (crashed during refcnt operations) */
  isDirty: Scalars['Boolean']['output'];
  /** Number of pending refcnt operations */
  pendingCount: Scalars['Int']['output'];
};

export type PoolUsage = {
  __typename?: 'PoolUsage';
  compressedSize: Scalars['BigInt']['output'];
  compressedSizeLastMonth: Scalars['BigInt']['output'];
  compressedSizeRange: Array<BigIntTimeSerie>;
  longestChain: Scalars['Int']['output'];
  longestChainLastMonth?: Maybe<Scalars['Int']['output']>;
  longestChainRange: Array<NumberTimeSerie>;
  nbChunk: Scalars['Int']['output'];
  nbChunkLastMonth?: Maybe<Scalars['Int']['output']>;
  nbChunkRange: Array<NumberTimeSerie>;
  nbRef: Scalars['Int']['output'];
  nbRefLastMonth?: Maybe<Scalars['Int']['output']>;
  nbRefRange: Array<NumberTimeSerie>;
  size: Scalars['BigInt']['output'];
  sizeLastMonth: Scalars['BigInt']['output'];
  sizeRange: Array<BigIntTimeSerie>;
  unusedSize: Scalars['BigInt']['output'];
  unusedSizeLastMonth: Scalars['BigInt']['output'];
  unusedSizeRange: Array<BigIntTimeSerie>;
};

export type QueryMerged = {
  __typename?: 'QueryMerged';
  /**
   * Configured archive profiles (from `archiving.yml`), read-only — used
   * to populate the manual archive-run trigger in the Tasks UI. There is
   * no mutation to create/edit profiles; that stays YAML-only.
   */
  archiveProfiles: Array<ArchiveProfile>;
  backup: BackupEx;
  backups: Array<BackupEx>;
  events: Array<ApplicationEvent>;
  /** Récupère tous les backups ayant échoué (avec error_message non null) */
  failedBackups: Array<BackupEx>;
  host: Host;
  hosts: Array<Host>;
  informations: ServerInformations;
  /**
   * Gets the health status of the storage pool.
   * Checks for dirty state (crashed refcnt operations).
   */
  poolHealth: PoolHealthStatusDto;
  queue: Array<Job>;
  queueStats: QueueStats;
  statistics: Statistics;
};


export type QueryMergedBackupArgs = {
  hostname: Scalars['String']['input'];
  id: Scalars['String']['input'];
};


export type QueryMergedBackupsArgs = {
  hostname: Scalars['String']['input'];
};


export type QueryMergedEventsArgs = {
  firstEvent: Scalars['DateTime']['input'];
  lastEvent: Scalars['DateTime']['input'];
  limit?: InputMaybe<Scalars['Int']['input']>;
  offset?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryMergedHostArgs = {
  hostname: Scalars['String']['input'];
};


export type QueryMergedQueueArgs = {
  input: QueueListInput;
};

export type QueueListInput = {
  operationName?: InputMaybe<Scalars['String']['input']>;
  queueName?: InputMaybe<Scalars['String']['input']>;
  state?: InputMaybe<JobStatus>;
};

export type QueueStats = {
  __typename?: 'QueueStats';
  dead: Scalars['Int']['output'];
  failed: Scalars['Int']['output'];
  lastExecution?: Maybe<Scalars['DateTime']['output']>;
  nextWakeup?: Maybe<Scalars['DateTime']['output']>;
  pending: Scalars['Int']['output'];
  running: Scalars['Int']['output'];
  success: Scalars['Int']['output'];
};

export type RefcntProgression = {
  __typename?: 'RefcntProgression';
  errorCount: Scalars['Int']['output'];
  progressCurrent: Scalars['Int']['output'];
  progressMax: Scalars['Int']['output'];
  totalCount: Scalars['Int']['output'];
};

export enum RemoveErrorState {
  AddReferencesToPoolError = 'ADD_REFERENCES_TO_POOL_ERROR',
  BackupRemovalError = 'BACKUP_REMOVAL_ERROR',
  RefcntRemovalError = 'REFCNT_REMOVAL_ERROR',
  Unknown = 'UNKNOWN'
}

export enum RemoveExecutionState {
  AddReferencesToPool = 'ADD_REFERENCES_TO_POOL',
  Completed = 'COMPLETED',
  RemovingBackup = 'REMOVING_BACKUP',
  RemovingRefcnt = 'REMOVING_REFCNT',
  Waiting = 'WAITING'
}

export enum RemovingStageDto {
  RemoveFromHost = 'REMOVE_FROM_HOST',
  ToRemove = 'TO_REMOVE',
  ToRemoveInPool = 'TO_REMOVE_IN_POOL'
}

export enum RestoreErrorState {
  AuthenticationError = 'AUTHENTICATION_ERROR',
  PreparationError = 'PREPARATION_ERROR',
  RestoreError = 'RESTORE_ERROR',
  Unknown = 'UNKNOWN'
}

export enum RestoreExecutionState {
  Authentication = 'AUTHENTICATION',
  Completed = 'COMPLETED',
  Preparation = 'PREPARATION',
  Restoring = 'RESTORING',
  Waiting = 'WAITING'
}

export type RestoreFilesInput = {
  selection: Array<Scalars['String']['input']>;
  share: Scalars['String']['input'];
};

export type RestoreInput = {
  destinationDirectory: Scalars['String']['input'];
  files: Array<RestoreFilesInput>;
  hostname: Scalars['String']['input'];
  /** UUID v7 of the backup to restore */
  id: Scalars['String']['input'];
};

/** Retention category DTO — mirrors [`woodstock::server::backup::retention::RetentionCategory`]. */
export enum RetentionCategoryDto {
  /** Representative of a daily slot. */
  Daily = 'DAILY',
  /** Representative of an hourly slot. */
  Hourly = 'HOURLY',
  /** Most recent terminal backup — protected from deletion. */
  LastBackup = 'LAST_BACKUP',
  /** Representative of a monthly slot. */
  Monthly = 'MONTHLY',
  /** Not retained by any slot — scheduled for deletion. */
  Surplus = 'SURPLUS',
  /** Representative of a weekly (ISO-week) slot. */
  Weekly = 'WEEKLY',
  /** Representative of a yearly slot. */
  Yearly = 'YEARLY'
}

export type Schedule = {
  __typename?: 'Schedule';
  activated?: Maybe<Scalars['Boolean']['output']>;
  backupPeriod?: Maybe<Scalars['Int']['output']>;
  backupToKeep?: Maybe<ScheduledBackupToKeep>;
};

export type ScheduledBackupToKeep = {
  __typename?: 'ScheduledBackupToKeep';
  daily?: Maybe<Scalars['Int']['output']>;
  hourly?: Maybe<Scalars['Int']['output']>;
  monthly?: Maybe<Scalars['Int']['output']>;
  weekly?: Maybe<Scalars['Int']['output']>;
  yearly?: Maybe<Scalars['Int']['output']>;
};

export type ServerInformations = {
  __typename?: 'ServerInformations';
  hostname: Scalars['String']['output'];
  uptime: Scalars['Int']['output'];
  woodstockVersion: Scalars['String']['output'];
};

export enum ShareExecutionState {
  Failed = 'FAILED',
  FileList = 'FILE_LIST',
  InProgress = 'IN_PROGRESS',
  Success = 'SUCCESS',
  Waiting = 'WAITING'
}

export type ShareState = {
  __typename?: 'ShareState';
  backupProgression: BackupProgression;
  executionState: ShareExecutionState;
  fileListProgression: FileListProgression;
  share: Scalars['String']['output'];
  snapshotFailureReason?: Maybe<Scalars['String']['output']>;
  snapshotMethod: SnapshotMethodDto;
};

export enum SnapshotMethodDto {
  Btrfs = 'BTRFS',
  None = 'NONE',
  Vss = 'VSS'
}

export type Statistics = {
  __typename?: 'Statistics';
  diskUsage: DiskUsage;
  hosts: Array<HostStatistics>;
  poolUsage: PoolUsage;
};

export type SubscriptionMerged = {
  __typename?: 'SubscriptionMerged';
  /**
   * Subscription: real backup changes for a given host.
   *
   * Emitted via Redis Pub/Sub (`woodstock:backup:changed`) whenever
   * `backup.yml` is written to disk by any server process
   * (job_worker, api_server, …):
   * - Creation or update (`add_or_replace_backup`, `update_backup`)
   * - Removal signalled (`remove_backup`)
   *
   * For completed removals (backup no longer on disk), the front-end
   * uses the `jobUpdated(kind: "remove")` subscription and triggers a refetch.
   */
  backupUpdated: BackupEx;
  /**
   * Subscription: job updates with optional `host` and `kind` filters.
   * Without filters, all jobs are returned (historical behaviour).
   */
  jobUpdated: Job;
};


export type SubscriptionMergedBackupUpdatedArgs = {
  hostname: Scalars['String']['input'];
};


export type SubscriptionMergedJobUpdatedArgs = {
  host?: InputMaybe<Scalars['String']['input']>;
  kind?: InputMaybe<Scalars['String']['input']>;
};

export type UnusedProgression = {
  __typename?: 'UnusedProgression';
  inNothing: Scalars['Int']['output'];
  inRefcnt: Scalars['Int']['output'];
  inUnused: Scalars['Int']['output'];
  missing: Scalars['Int']['output'];
  progressCurrent: Scalars['Int']['output'];
  progressMax: Scalars['Int']['output'];
};

export type ApplicationEventFragment = { __typename?: 'ApplicationEvent', uuid: string, type: EventType, step: EventStep, source: EventSource, timestamp: Date, errorMessages: Array<string>, status: EventStatus, information?:
    | (
      { __typename: 'EventBackupInformation' }
      & { ' $fragmentRefs'?: { 'EventBackupInformationFragment': EventBackupInformationFragment } }
    )
    | (
      { __typename: 'EventHashConversionInformation' }
      & { ' $fragmentRefs'?: { 'EventHashConversionInformationFragment': EventHashConversionInformationFragment } }
    )
    | (
      { __typename: 'EventPoolCleanedInformation' }
      & { ' $fragmentRefs'?: { 'EventPoolCleanedInformationFragment': EventPoolCleanedInformationFragment } }
    )
    | (
      { __typename: 'EventPoolInformation' }
      & { ' $fragmentRefs'?: { 'EventPoolInformationFragment': EventPoolInformationFragment } }
    )
   | null } & { ' $fragmentName'?: 'ApplicationEventFragment' };

export type EventBackupInformationFragment = { __typename?: 'EventBackupInformation', hostname: string, number: number, sharePath: Array<string> } & { ' $fragmentName'?: 'EventBackupInformationFragment' };

export type EventPoolInformationFragment = { __typename?: 'EventPoolInformation', fix: boolean, refcount: number, refcountError: number, inUnused: number, inRefcnt: number, inNothing: number, missing: number, chunkCount: number, chunkError: number } & { ' $fragmentName'?: 'EventPoolInformationFragment' };

export type EventPoolCleanedInformationFragment = { __typename?: 'EventPoolCleanedInformation', size: bigint, count: number } & { ' $fragmentName'?: 'EventPoolCleanedInformationFragment' };

export type EventHashConversionInformationFragment = { __typename?: 'EventHashConversionInformation', count: number, algorithm: ChunkAlgorithm } & { ' $fragmentName'?: 'EventHashConversionInformationFragment' };

export type PoolHealthQueryVariables = Exact<{ [key: string]: never; }>;


export type PoolHealthQuery = { __typename?: 'QueryMerged', poolHealth: { __typename?: 'PoolHealthStatusDto', healthy: boolean, isDirty: boolean, pendingCount: number } };

export type ArchiveProfilesQueryVariables = Exact<{ [key: string]: never; }>;


export type ArchiveProfilesQuery = { __typename?: 'QueryMerged', archiveProfiles: Array<{ __typename?: 'ArchiveProfile', name: string, enabled: boolean, format: ArchiveFormat, destination: string, scheduleCron: string, checksum: boolean, compressionLevel?: number | null, hostSelectionMode: HostSelectionMode, hostSelectionPattern?: string | null, hostSelectionHosts?: Array<string> | null }> };

export type RunArchiveMutationVariables = Exact<{
  profile: Scalars['String']['input'];
  host?: InputMaybe<Scalars['String']['input']>;
}>;


export type RunArchiveMutation = { __typename?: 'MutationRoot', runArchive: { __typename?: 'ArchiveRunResponse', jobIds: Array<string> } };

export type HostQueryVariables = Exact<{
  hostname: Scalars['String']['input'];
}>;


export type HostQuery = { __typename?: 'QueryMerged', host: { __typename?: 'Host', name: string, agentVersion?: string | null, availibilityState?: HostAvailibilityState | null, timeSinceLastBackup?: number | null, dateToNextBackup?: Date | null, addresses?: Array<string> | null, lastBackup?: { __typename?: 'BackupEx', agentVersion?: string | null, status: (
        { __typename?: 'BackupStatusDto' }
        & { ' $fragmentRefs'?: { 'BackupStatusFieldsFragment': BackupStatusFieldsFragment } }
      ) } | null, configuration: { __typename?: 'HostConfiguration', operations: { __typename?: 'HostConfigOperation', preCommands?: Array<{ __typename?: 'ExecuteCommandOperation', command: string }> | null, operation?: { __typename?: 'BackupOperation', shares: Array<{ __typename?: 'BackupTaskShare', name: string }> } | null, postCommands?: Array<{ __typename?: 'ExecuteCommandOperation', command: string }> | null }, schedule?: { __typename?: 'Schedule', activated?: boolean | null } | null } } };

export type HostsQueryVariables = Exact<{ [key: string]: never; }>;


export type HostsQuery = { __typename?: 'QueryMerged', hosts: Array<{ __typename?: 'Host', name: string, agentVersion?: string | null, availibilityState?: HostAvailibilityState | null, timeSinceLastBackup?: number | null, dateToNextBackup?: Date | null, lastBackup?: { __typename?: 'BackupEx', number: number, startDate: Date, fileSize: bigint, agentVersion?: string | null, status: (
        { __typename?: 'BackupStatusDto' }
        & { ' $fragmentRefs'?: { 'BackupStatusFieldsFragment': BackupStatusFieldsFragment } }
      ) } | null, configuration: { __typename?: 'HostConfiguration', schedule?: { __typename?: 'Schedule', activated?: boolean | null } | null } }> };

export type BackupQueryVariables = Exact<{
  hostname: Scalars['String']['input'];
  id: Scalars['String']['input'];
}>;


export type BackupQuery = { __typename?: 'QueryMerged', backup: { __typename?: 'BackupEx', id: string, number: number, startDate: Date, endDate?: Date | null, errorCount: number, fileCount: number, newFileCount: number, existingFileCount: number, removedFileCount: number, modifiedFileCount: number, fileSize: bigint, newFileSize: bigint, existingFileSize: bigint, speed: number, status: (
      { __typename?: 'BackupStatusDto' }
      & { ' $fragmentRefs'?: { 'BackupStatusFieldsFragment': BackupStatusFieldsFragment } }
    ), shareRecords: Array<{ __typename?: 'BackupShareRecord', path: string, snapshotMethod: SnapshotMethodDto, snapshotFailureReason?: string | null }> } };

export type BackupsQueryVariables = Exact<{
  hostname: Scalars['String']['input'];
}>;


export type BackupsQuery = { __typename?: 'QueryMerged', backups: Array<{ __typename?: 'BackupEx', id: string, number: number, retentionCategory?: RetentionCategoryDto | null, startDate: Date, endDate?: Date | null, errorCount: number, fileCount: number, newFileCount: number, existingFileCount: number, removedFileCount: number, modifiedFileCount: number, fileSize: bigint, newFileSize: bigint, existingFileSize: bigint, speed: number, status: (
      { __typename?: 'BackupStatusDto' }
      & { ' $fragmentRefs'?: { 'BackupStatusFieldsFragment': BackupStatusFieldsFragment } }
    ) }> };

export type BackupsBrowseQueryVariables = Exact<{
  hostname: Scalars['String']['input'];
  id: Scalars['String']['input'];
  sharePath: Scalars['String']['input'];
  path: Scalars['Buffer']['input'];
}>;


export type BackupsBrowseQuery = { __typename?: 'QueryMerged', backup: { __typename?: 'BackupEx', id: string, files: Array<(
      { __typename?: 'FileDescription' }
      & { ' $fragmentRefs'?: { 'FragmentFileDescriptionFragment': FragmentFileDescriptionFragment } }
    )> } };

export type CreateBackupMutationVariables = Exact<{
  hostname: Scalars['String']['input'];
}>;


export type CreateBackupMutation = { __typename?: 'MutationRoot', createBackup: { __typename?: 'JobResponse', id: string } };

export type PurgeRetentionMutationVariables = Exact<{
  hostname: Scalars['String']['input'];
}>;


export type PurgeRetentionMutation = { __typename?: 'MutationRoot', purgeRetention: { __typename?: 'JobResponse', id: string } };

export type RemoveBackupMutationVariables = Exact<{
  hostname: Scalars['String']['input'];
  id: Scalars['String']['input'];
}>;


export type RemoveBackupMutation = { __typename?: 'MutationRoot', removeBackup: { __typename?: 'JobResponse', id: string } };

export type BackupStatusFieldsFragment = { __typename?: 'BackupStatusDto', statusType: BackupStatusTypeDto, finishingStage?: FinishingStageDto | null, abortingStage?: AbortingStageDto | null, failedStage?: FailedStageDto | null, removingStage?: RemovingStageDto | null } & { ' $fragmentName'?: 'BackupStatusFieldsFragment' };

export type FragmentFileDescriptionFragment = { __typename?: 'FileDescription', path: string, type: FileManifestTypeDto, symlink: string, stats?: { __typename?: 'FileStat', ownerId: number, groupId: number, mode: number, size: bigint, lastModified: number } | null } & { ' $fragmentName'?: 'FragmentFileDescriptionFragment' };

export type SharesBrowseQueryVariables = Exact<{
  hostname: Scalars['String']['input'];
  id: Scalars['String']['input'];
}>;


export type SharesBrowseQuery = { __typename?: 'QueryMerged', backup: { __typename?: 'BackupEx', id: string, shares: Array<(
      { __typename?: 'FileDescription' }
      & { ' $fragmentRefs'?: { 'FragmentFileDescriptionFragment': FragmentFileDescriptionFragment } }
    )> } };

export type BackupUpdatedSubscriptionVariables = Exact<{
  hostname: Scalars['String']['input'];
}>;


export type BackupUpdatedSubscription = { __typename?: 'SubscriptionMerged', backupUpdated: { __typename?: 'BackupEx', id: string, number: number, startDate: Date, endDate?: Date | null, errorCount: number, fileCount: number, newFileCount: number, existingFileCount: number, removedFileCount: number, modifiedFileCount: number, fileSize: bigint, newFileSize: bigint, existingFileSize: bigint, speed: number, status: { __typename?: 'BackupStatusDto', statusType: BackupStatusTypeDto, finishingStage?: FinishingStageDto | null, abortingStage?: AbortingStageDto | null, failedStage?: FailedStageDto | null, removingStage?: RemovingStageDto | null } } };

export type JobRemoveUpdatedSubscriptionVariables = Exact<{
  host?: InputMaybe<Scalars['String']['input']>;
  kind?: InputMaybe<Scalars['String']['input']>;
}>;


export type JobRemoveUpdatedSubscription = { __typename?: 'SubscriptionMerged', jobUpdated: { __typename?: 'Job', jobId: string, status: JobStatus } };

export type JobBackupUpdatedSubscriptionVariables = Exact<{
  host?: InputMaybe<Scalars['String']['input']>;
  kind?: InputMaybe<Scalars['String']['input']>;
}>;


export type JobBackupUpdatedSubscription = { __typename?: 'SubscriptionMerged', jobUpdated: { __typename?: 'Job', jobId: string, status: JobStatus } };

export type JobPoolResponseFragment = { __typename?: 'JobResponse', id: string } & { ' $fragmentName'?: 'JobPoolResponseFragment' };

export type RestoreBackupMutationVariables = Exact<{
  input: RestoreInput;
}>;


export type RestoreBackupMutation = { __typename?: 'MutationRoot', restoreBackup: (
    { __typename?: 'JobResponse' }
    & { ' $fragmentRefs'?: { 'JobPoolResponseFragment': JobPoolResponseFragment } }
  ) };

export type ClearCacheMutationVariables = Exact<{ [key: string]: never; }>;


export type ClearCacheMutation = { __typename?: 'MutationRoot', clearCache: { __typename?: 'JobResponse', id: string } };

export type EventsQueryVariables = Exact<{
  firstEvent: Scalars['DateTime']['input'];
  lastEvent: Scalars['DateTime']['input'];
  limit?: InputMaybe<Scalars['Int']['input']>;
  offset?: InputMaybe<Scalars['Int']['input']>;
}>;


export type EventsQuery = { __typename?: 'QueryMerged', events: Array<(
    { __typename?: 'ApplicationEvent' }
    & { ' $fragmentRefs'?: { 'ApplicationEventFragment': ApplicationEventFragment } }
  )> };

export type CleanupPoolMutationVariables = Exact<{ [key: string]: never; }>;


export type CleanupPoolMutation = { __typename?: 'MutationRoot', cleanupPool: (
    { __typename?: 'JobResponse' }
    & { ' $fragmentRefs'?: { 'JobPoolResponseFragment': JobPoolResponseFragment } }
  ) };

export type CheckAndFixPoolMutationVariables = Exact<{
  fix: Scalars['Boolean']['input'];
  verifyChunks: Scalars['Boolean']['input'];
}>;


export type CheckAndFixPoolMutation = { __typename?: 'MutationRoot', checkAndFixPool: (
    { __typename?: 'JobResponse' }
    & { ' $fragmentRefs'?: { 'JobPoolResponseFragment': JobPoolResponseFragment } }
  ) };

export type ServerInformationsQueryVariables = Exact<{ [key: string]: never; }>;


export type ServerInformationsQuery = { __typename?: 'QueryMerged', informations: { __typename?: 'ServerInformations', uptime: number, hostname: string, woodstockVersion: string } };

export type DiskUsageStatisticsQueryVariables = Exact<{ [key: string]: never; }>;


export type DiskUsageStatisticsQuery = { __typename?: 'QueryMerged', statistics: { __typename?: 'Statistics', hosts: Array<{ __typename?: 'HostStatistics', host: string, size: bigint, compressedSize: bigint }> } };

export type QueueStatisticsQueryVariables = Exact<{ [key: string]: never; }>;


export type QueueStatisticsQuery = { __typename?: 'QueryMerged', queueStats: { __typename?: 'QueueStats', pending: number, running: number, success: number, failed: number, dead: number } };

export type PoolStatisticsQueryVariables = Exact<{ [key: string]: never; }>;


export type PoolStatisticsQuery = { __typename?: 'QueryMerged', statistics: { __typename?: 'Statistics', diskUsage: { __typename?: 'DiskUsage', used: bigint, usedLastMonth: bigint, free: bigint, total: bigint }, poolUsage: { __typename?: 'PoolUsage', nbChunk: number, nbChunkLastMonth?: number | null, nbRef: number, nbRefLastMonth?: number | null, size: bigint, compressedSize: bigint, compressedSizeLastMonth: bigint, unusedSize: bigint, nbChunkRange: Array<{ __typename?: 'NumberTimeSerie', time: Date, value: number }>, compressedSizeRange: Array<{ __typename?: 'BigIntTimeSerie', time: Date, value: bigint }> } } };

export type JobBackupDataFragment = { __typename?: 'JobBackupData', host: string, number: number, ip?: string | null, startDate?: Date | null } & { ' $fragmentName'?: 'JobBackupDataFragment' };

export type JobRestoreDataFragment = { __typename?: 'JobRestoreData', host: string, number: number, ip?: string | null, startDate?: Date | null, destinationDirectory: string, files: Array<{ __typename?: 'JobRestoreDataSelection', share: string, selection: Array<string> }> } & { ' $fragmentName'?: 'JobRestoreDataFragment' };

export type JobRemoveDataFragment = { __typename?: 'JobRemoveData', host: string, number: number, startDate?: Date | null } & { ' $fragmentName'?: 'JobRemoveDataFragment' };

export type JobCleanupDataFragment = { __typename?: 'JobCleanupData', target?: string | null } & { ' $fragmentName'?: 'JobCleanupDataFragment' };

export type JobFsckDataFragment = { __typename?: 'JobFsckData', dryRun: boolean, verifyChunks: boolean } & { ' $fragmentName'?: 'JobFsckDataFragment' };

export type JobArchiveDataFragment = { __typename?: 'JobArchiveData', profileName: string, hostnames: Array<string> } & { ' $fragmentName'?: 'JobArchiveDataFragment' };

export type BackupTaskStateFragment = { __typename?: 'JobBackupTaskState', backupExecutionState: BackupExecutionState, backupErrorState?: BackupErrorState | null, backupErrorMessage?: string | null, globalProgression: { __typename?: 'BackupProgression', startDate: Date, startTransferDate?: Date | null, endTransferDate?: Date | null, fileSize: bigint, newFileSize: bigint, modifiedFileSize: bigint, compressedFileSize: bigint, newCompressedFileSize: bigint, modifiedCompressedFileSize: bigint, fileCount: number, newFileCount: number, modifiedFileCount: number, removedFileCount: number, errorCount: number, speed: number, percent: number, progressCurrent: bigint, progressMax: bigint }, preCommandStates: Array<{ __typename?: 'ExecuteCommandState', executionState: ExecuteCommandExecutionState, command: { __typename?: 'ExecuteCommandOperation', command: string } }>, shareStates: Array<{ __typename?: 'ShareState', share: string, executionState: ShareExecutionState, backupProgression: { __typename?: 'BackupProgression', startDate: Date, startTransferDate?: Date | null, endTransferDate?: Date | null, fileSize: bigint, newFileSize: bigint, modifiedFileSize: bigint, compressedFileSize: bigint, newCompressedFileSize: bigint, modifiedCompressedFileSize: bigint, fileCount: number, newFileCount: number, modifiedFileCount: number, removedFileCount: number, errorCount: number, speed: number, percent: number, progressCurrent: bigint, progressMax: bigint }, fileListProgression: { __typename?: 'FileListProgression', fileSize: bigint, newFileSize: bigint, modifiedFileSize: bigint, newFileCount: number, modifiedFileCount: number, removedFileCount: number } }>, postCommandStates: Array<{ __typename?: 'ExecuteCommandState', executionState: ExecuteCommandExecutionState, command: { __typename?: 'ExecuteCommandOperation', command: string } }> } & { ' $fragmentName'?: 'BackupTaskStateFragment' };

export type RestoreTaskStateFragment = { __typename?: 'JobRestoreTaskState', restoreExecutionState: RestoreExecutionState, restoreErrorState?: RestoreErrorState | null, restoreErrorMessage?: string | null, restoreProgression: { __typename?: 'BackupProgression', startDate: Date, startTransferDate?: Date | null, endTransferDate?: Date | null, fileSize: bigint, newFileSize: bigint, modifiedFileSize: bigint, compressedFileSize: bigint, newCompressedFileSize: bigint, modifiedCompressedFileSize: bigint, fileCount: number, newFileCount: number, modifiedFileCount: number, removedFileCount: number, errorCount: number, speed: number, percent: number, progressCurrent: bigint, progressMax: bigint } } & { ' $fragmentName'?: 'RestoreTaskStateFragment' };

export type RemoveTaskStateFragment = { __typename?: 'JobRemoveState', removeExecutionState: RemoveExecutionState, removeErrorState?: RemoveErrorState | null, removeErrorMessage?: string | null } & { ' $fragmentName'?: 'RemoveTaskStateFragment' };

export type CleanerTaskStateFragment = { __typename?: 'JobCleanerTaskState', cleanerExecutionState: CleanerExecutionState, cleanerErrorState?: CleanerErrorState | null, cleanerErrorMessage?: string | null, cleanerProgress: { __typename?: 'CleanerProgression', progressMax: number, progressCurrent: number, fileSize: bigint, compressedFileSize: bigint } } & { ' $fragmentName'?: 'CleanerTaskStateFragment' };

export type FsckTaskStateFragment = { __typename?: 'JobFsckTaskState', dryRun: boolean, fsckExecutionState: FsckExecutionState, fsckErrorState?: FsckErrorState | null, fsckErrorMessage?: string | null, refcntProgression: { __typename?: 'RefcntProgression', progressMax: number, progressCurrent: number, errorCount: number, totalCount: number }, unusedProgression: { __typename?: 'UnusedProgression', progressMax: number, progressCurrent: number, inNothing: number, inRefcnt: number, inUnused: number, missing: number }, chunkProgression: { __typename?: 'ChunkProgression', progressMax: number, progressCurrent: number, errorCount: number, totalCount: number } } & { ' $fragmentName'?: 'FsckTaskStateFragment' };

export type ArchiveTaskStateFragment = { __typename?: 'JobArchiveTaskState', currentHost?: string | null, hostsDone: number, hostsTotal: number, progressCurrent: bigint, progressMax: bigint, percent: number, fileCount: number, archiveSize: bigint, speed: number, failedHosts: Array<string>, hostStates: Array<{ __typename?: 'ArchiveHostState', hostname: string, executionState: ArchiveHostExecutionState, progressCurrent: bigint, progressMax: bigint, percent: number, fileCount: number, archiveSize?: bigint | null }> } & { ' $fragmentName'?: 'ArchiveTaskStateFragment' };

export type JobFragment = { __typename?: 'Job', jobId: string, kind: JobKind, status: JobStatus, timestamp: number, host?: string | null, failedReason?: string | null, data:
    | (
      { __typename?: 'JobArchiveData' }
      & { ' $fragmentRefs'?: { 'JobArchiveDataFragment': JobArchiveDataFragment } }
    )
    | (
      { __typename?: 'JobBackupData' }
      & { ' $fragmentRefs'?: { 'JobBackupDataFragment': JobBackupDataFragment } }
    )
    | (
      { __typename?: 'JobCleanupData' }
      & { ' $fragmentRefs'?: { 'JobCleanupDataFragment': JobCleanupDataFragment } }
    )
    | (
      { __typename?: 'JobFsckData' }
      & { ' $fragmentRefs'?: { 'JobFsckDataFragment': JobFsckDataFragment } }
    )
    | (
      { __typename?: 'JobRemoveData' }
      & { ' $fragmentRefs'?: { 'JobRemoveDataFragment': JobRemoveDataFragment } }
    )
    | (
      { __typename?: 'JobRestoreData' }
      & { ' $fragmentRefs'?: { 'JobRestoreDataFragment': JobRestoreDataFragment } }
    )
    | { __typename?: 'JobStatsData' }
  , progress?:
    | (
      { __typename?: 'JobArchiveTaskState' }
      & { ' $fragmentRefs'?: { 'ArchiveTaskStateFragment': ArchiveTaskStateFragment } }
    )
    | (
      { __typename?: 'JobBackupTaskState' }
      & { ' $fragmentRefs'?: { 'BackupTaskStateFragment': BackupTaskStateFragment } }
    )
    | (
      { __typename?: 'JobCleanerTaskState' }
      & { ' $fragmentRefs'?: { 'CleanerTaskStateFragment': CleanerTaskStateFragment } }
    )
    | (
      { __typename?: 'JobFsckTaskState' }
      & { ' $fragmentRefs'?: { 'FsckTaskStateFragment': FsckTaskStateFragment } }
    )
    | (
      { __typename?: 'JobRemoveState' }
      & { ' $fragmentRefs'?: { 'RemoveTaskStateFragment': RemoveTaskStateFragment } }
    )
    | (
      { __typename?: 'JobRestoreTaskState' }
      & { ' $fragmentRefs'?: { 'RestoreTaskStateFragment': RestoreTaskStateFragment } }
    )
   | null } & { ' $fragmentName'?: 'JobFragment' };

export type TasksQueryVariables = Exact<{
  input: QueueListInput;
}>;


export type TasksQuery = { __typename?: 'QueryMerged', queue: Array<(
    { __typename?: 'Job' }
    & { ' $fragmentRefs'?: { 'JobFragment': JobFragment } }
  )> };

export type QueueTasksJobUpdatedSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type QueueTasksJobUpdatedSubscription = { __typename?: 'SubscriptionMerged', jobUpdated: (
    { __typename?: 'Job' }
    & { ' $fragmentRefs'?: { 'JobFragment': JobFragment } }
  ) };

export const EventBackupInformationFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EventBackupInformation"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventBackupInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"hostname"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"sharePath"}}]}}]} as unknown as DocumentNode<EventBackupInformationFragment, unknown>;
export const EventPoolInformationFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EventPoolInformation"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventPoolInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"fix"}},{"kind":"Field","name":{"kind":"Name","value":"refcount"}},{"kind":"Field","name":{"kind":"Name","value":"refcountError"}},{"kind":"Field","name":{"kind":"Name","value":"inUnused"}},{"kind":"Field","name":{"kind":"Name","value":"inRefcnt"}},{"kind":"Field","name":{"kind":"Name","value":"inNothing"}},{"kind":"Field","name":{"kind":"Name","value":"missing"}},{"kind":"Field","name":{"kind":"Name","value":"chunkCount"}},{"kind":"Field","name":{"kind":"Name","value":"chunkError"}}]}}]} as unknown as DocumentNode<EventPoolInformationFragment, unknown>;
export const EventPoolCleanedInformationFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EventPoolCleanedInformation"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventPoolCleanedInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"count"}}]}}]} as unknown as DocumentNode<EventPoolCleanedInformationFragment, unknown>;
export const EventHashConversionInformationFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EventHashConversionInformation"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventHashConversionInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"count"}},{"kind":"Field","name":{"kind":"Name","value":"algorithm"}}]}}]} as unknown as DocumentNode<EventHashConversionInformationFragment, unknown>;
export const ApplicationEventFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ApplicationEvent"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"ApplicationEvent"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"uuid"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"step"}},{"kind":"Field","name":{"kind":"Name","value":"source"}},{"kind":"Field","name":{"kind":"Name","value":"timestamp"}},{"kind":"Field","name":{"kind":"Name","value":"errorMessages"}},{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"information"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventBackupInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"EventBackupInformation"}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventPoolInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"EventPoolInformation"}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventPoolCleanedInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"EventPoolCleanedInformation"}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventHashConversionInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"EventHashConversionInformation"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EventBackupInformation"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventBackupInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"hostname"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"sharePath"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EventPoolInformation"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventPoolInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"fix"}},{"kind":"Field","name":{"kind":"Name","value":"refcount"}},{"kind":"Field","name":{"kind":"Name","value":"refcountError"}},{"kind":"Field","name":{"kind":"Name","value":"inUnused"}},{"kind":"Field","name":{"kind":"Name","value":"inRefcnt"}},{"kind":"Field","name":{"kind":"Name","value":"inNothing"}},{"kind":"Field","name":{"kind":"Name","value":"missing"}},{"kind":"Field","name":{"kind":"Name","value":"chunkCount"}},{"kind":"Field","name":{"kind":"Name","value":"chunkError"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EventPoolCleanedInformation"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventPoolCleanedInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"count"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EventHashConversionInformation"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventHashConversionInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"count"}},{"kind":"Field","name":{"kind":"Name","value":"algorithm"}}]}}]} as unknown as DocumentNode<ApplicationEventFragment, unknown>;
export const BackupStatusFieldsFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BackupStatusFields"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"BackupStatusDto"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"statusType"}},{"kind":"Field","name":{"kind":"Name","value":"finishingStage"}},{"kind":"Field","name":{"kind":"Name","value":"abortingStage"}},{"kind":"Field","name":{"kind":"Name","value":"failedStage"}},{"kind":"Field","name":{"kind":"Name","value":"removingStage"}}]}}]} as unknown as DocumentNode<BackupStatusFieldsFragment, unknown>;
export const FragmentFileDescriptionFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"FragmentFileDescription"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"FileDescription"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"path"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"stats"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"ownerId"}},{"kind":"Field","name":{"kind":"Name","value":"groupId"}},{"kind":"Field","name":{"kind":"Name","value":"mode"}},{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"lastModified"}}]}},{"kind":"Field","name":{"kind":"Name","value":"symlink"}}]}}]} as unknown as DocumentNode<FragmentFileDescriptionFragment, unknown>;
export const JobPoolResponseFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobPoolResponse"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobResponse"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]} as unknown as DocumentNode<JobPoolResponseFragment, unknown>;
export const JobBackupDataFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobBackupData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobBackupData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"ip"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}}]}}]} as unknown as DocumentNode<JobBackupDataFragment, unknown>;
export const JobRestoreDataFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobRestoreData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobRestoreData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"ip"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"destinationDirectory"}},{"kind":"Field","name":{"kind":"Name","value":"files"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"share"}},{"kind":"Field","name":{"kind":"Name","value":"selection"}}]}}]}}]} as unknown as DocumentNode<JobRestoreDataFragment, unknown>;
export const JobRemoveDataFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobRemoveData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobRemoveData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}}]}}]} as unknown as DocumentNode<JobRemoveDataFragment, unknown>;
export const JobCleanupDataFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobCleanupData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobCleanupData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"target"}}]}}]} as unknown as DocumentNode<JobCleanupDataFragment, unknown>;
export const JobFsckDataFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobFsckData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobFsckData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"dryRun"}},{"kind":"Field","name":{"kind":"Name","value":"verifyChunks"}}]}}]} as unknown as DocumentNode<JobFsckDataFragment, unknown>;
export const JobArchiveDataFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobArchiveData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobArchiveData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"profileName"}},{"kind":"Field","name":{"kind":"Name","value":"hostnames"}}]}}]} as unknown as DocumentNode<JobArchiveDataFragment, unknown>;
export const BackupTaskStateFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BackupTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobBackupTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"backupExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"backupErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"backupErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","alias":{"kind":"Name","value":"globalProgression"},"name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"startTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"endTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}}]}},{"kind":"Field","name":{"kind":"Name","value":"preCommandStates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"executionState"}},{"kind":"Field","name":{"kind":"Name","value":"command"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"command"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"shareStates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"share"}},{"kind":"Field","name":{"kind":"Name","value":"executionState"}},{"kind":"Field","name":{"kind":"Name","value":"backupProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"startTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"endTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}}]}},{"kind":"Field","name":{"kind":"Name","value":"fileListProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"postCommandStates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"executionState"}},{"kind":"Field","name":{"kind":"Name","value":"command"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"command"}}]}}]}}]}}]} as unknown as DocumentNode<BackupTaskStateFragment, unknown>;
export const RestoreTaskStateFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RestoreTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobRestoreTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"restoreExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"restoreErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"restoreErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","alias":{"kind":"Name","value":"restoreProgression"},"name":{"kind":"Name","value":"globalProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"startTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"endTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}}]}}]}}]} as unknown as DocumentNode<RestoreTaskStateFragment, unknown>;
export const RemoveTaskStateFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RemoveTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobRemoveState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"removeExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"removeErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"removeErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}}]}}]} as unknown as DocumentNode<RemoveTaskStateFragment, unknown>;
export const CleanerTaskStateFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"CleanerTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobCleanerTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"cleanerExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"cleanerErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"cleanerErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","alias":{"kind":"Name","value":"cleanerProgress"},"name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}}]}}]}}]} as unknown as DocumentNode<CleanerTaskStateFragment, unknown>;
export const FsckTaskStateFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"FsckTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobFsckTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"fsckExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"fsckErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"fsckErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","name":{"kind":"Name","value":"dryRun"}},{"kind":"Field","name":{"kind":"Name","value":"refcntProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"totalCount"}}]}},{"kind":"Field","name":{"kind":"Name","value":"unusedProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"inNothing"}},{"kind":"Field","name":{"kind":"Name","value":"inRefcnt"}},{"kind":"Field","name":{"kind":"Name","value":"inUnused"}},{"kind":"Field","name":{"kind":"Name","value":"missing"}}]}},{"kind":"Field","name":{"kind":"Name","value":"chunkProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"totalCount"}}]}}]}}]} as unknown as DocumentNode<FsckTaskStateFragment, unknown>;
export const ArchiveTaskStateFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ArchiveTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobArchiveTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"currentHost"}},{"kind":"Field","name":{"kind":"Name","value":"hostsDone"}},{"kind":"Field","name":{"kind":"Name","value":"hostsTotal"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"archiveSize"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"failedHosts"}},{"kind":"Field","name":{"kind":"Name","value":"hostStates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"hostname"}},{"kind":"Field","name":{"kind":"Name","value":"executionState"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"archiveSize"}}]}}]}}]} as unknown as DocumentNode<ArchiveTaskStateFragment, unknown>;
export const JobFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Job"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Job"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"jobId"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"timestamp"}},{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"failedReason"}},{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobBackupData"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobRestoreData"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobRemoveData"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobCleanupData"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobFsckData"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobArchiveData"}}]}},{"kind":"Field","name":{"kind":"Name","value":"progress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BackupTaskState"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"RestoreTaskState"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"RemoveTaskState"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"CleanerTaskState"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"FsckTaskState"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"ArchiveTaskState"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobBackupData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobBackupData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"ip"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobRestoreData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobRestoreData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"ip"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"destinationDirectory"}},{"kind":"Field","name":{"kind":"Name","value":"files"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"share"}},{"kind":"Field","name":{"kind":"Name","value":"selection"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobRemoveData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobRemoveData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobCleanupData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobCleanupData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"target"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobFsckData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobFsckData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"dryRun"}},{"kind":"Field","name":{"kind":"Name","value":"verifyChunks"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobArchiveData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobArchiveData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"profileName"}},{"kind":"Field","name":{"kind":"Name","value":"hostnames"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BackupTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobBackupTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"backupExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"backupErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"backupErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","alias":{"kind":"Name","value":"globalProgression"},"name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"startTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"endTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}}]}},{"kind":"Field","name":{"kind":"Name","value":"preCommandStates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"executionState"}},{"kind":"Field","name":{"kind":"Name","value":"command"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"command"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"shareStates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"share"}},{"kind":"Field","name":{"kind":"Name","value":"executionState"}},{"kind":"Field","name":{"kind":"Name","value":"backupProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"startTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"endTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}}]}},{"kind":"Field","name":{"kind":"Name","value":"fileListProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"postCommandStates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"executionState"}},{"kind":"Field","name":{"kind":"Name","value":"command"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"command"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RestoreTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobRestoreTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"restoreExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"restoreErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"restoreErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","alias":{"kind":"Name","value":"restoreProgression"},"name":{"kind":"Name","value":"globalProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"startTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"endTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RemoveTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobRemoveState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"removeExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"removeErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"removeErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"CleanerTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobCleanerTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"cleanerExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"cleanerErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"cleanerErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","alias":{"kind":"Name","value":"cleanerProgress"},"name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"FsckTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobFsckTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"fsckExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"fsckErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"fsckErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","name":{"kind":"Name","value":"dryRun"}},{"kind":"Field","name":{"kind":"Name","value":"refcntProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"totalCount"}}]}},{"kind":"Field","name":{"kind":"Name","value":"unusedProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"inNothing"}},{"kind":"Field","name":{"kind":"Name","value":"inRefcnt"}},{"kind":"Field","name":{"kind":"Name","value":"inUnused"}},{"kind":"Field","name":{"kind":"Name","value":"missing"}}]}},{"kind":"Field","name":{"kind":"Name","value":"chunkProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"totalCount"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ArchiveTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobArchiveTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"currentHost"}},{"kind":"Field","name":{"kind":"Name","value":"hostsDone"}},{"kind":"Field","name":{"kind":"Name","value":"hostsTotal"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"archiveSize"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"failedHosts"}},{"kind":"Field","name":{"kind":"Name","value":"hostStates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"hostname"}},{"kind":"Field","name":{"kind":"Name","value":"executionState"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"archiveSize"}}]}}]}}]} as unknown as DocumentNode<JobFragment, unknown>;
export const PoolHealthDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"PoolHealth"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"poolHealth"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"healthy"}},{"kind":"Field","name":{"kind":"Name","value":"isDirty"}},{"kind":"Field","name":{"kind":"Name","value":"pendingCount"}}]}}]}}]} as unknown as DocumentNode<PoolHealthQuery, PoolHealthQueryVariables>;
export const ArchiveProfilesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"ArchiveProfiles"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"archiveProfiles"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"enabled"}},{"kind":"Field","name":{"kind":"Name","value":"format"}},{"kind":"Field","name":{"kind":"Name","value":"destination"}},{"kind":"Field","name":{"kind":"Name","value":"scheduleCron"}},{"kind":"Field","name":{"kind":"Name","value":"checksum"}},{"kind":"Field","name":{"kind":"Name","value":"compressionLevel"}},{"kind":"Field","name":{"kind":"Name","value":"hostSelectionMode"}},{"kind":"Field","name":{"kind":"Name","value":"hostSelectionPattern"}},{"kind":"Field","name":{"kind":"Name","value":"hostSelectionHosts"}}]}}]}}]} as unknown as DocumentNode<ArchiveProfilesQuery, ArchiveProfilesQueryVariables>;
export const RunArchiveDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"runArchive"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"profile"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"host"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"runArchive"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"profile"},"value":{"kind":"Variable","name":{"kind":"Name","value":"profile"}}},{"kind":"Argument","name":{"kind":"Name","value":"host"},"value":{"kind":"Variable","name":{"kind":"Name","value":"host"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"jobIds"}}]}}]}}]} as unknown as DocumentNode<RunArchiveMutation, RunArchiveMutationVariables>;
export const HostDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"Host"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"hostname"},"value":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"agentVersion"}},{"kind":"Field","name":{"kind":"Name","value":"lastBackup"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"agentVersion"}},{"kind":"Field","name":{"kind":"Name","value":"status"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BackupStatusFields"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"availibilityState"}},{"kind":"Field","name":{"kind":"Name","value":"timeSinceLastBackup"}},{"kind":"Field","name":{"kind":"Name","value":"dateToNextBackup"}},{"kind":"Field","name":{"kind":"Name","value":"addresses"}},{"kind":"Field","name":{"kind":"Name","value":"configuration"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"operations"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"preCommands"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"command"}}]}},{"kind":"Field","name":{"kind":"Name","value":"operation"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"shares"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"postCommands"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"command"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"schedule"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"activated"}}]}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BackupStatusFields"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"BackupStatusDto"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"statusType"}},{"kind":"Field","name":{"kind":"Name","value":"finishingStage"}},{"kind":"Field","name":{"kind":"Name","value":"abortingStage"}},{"kind":"Field","name":{"kind":"Name","value":"failedStage"}},{"kind":"Field","name":{"kind":"Name","value":"removingStage"}}]}}]} as unknown as DocumentNode<HostQuery, HostQueryVariables>;
export const HostsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"Hosts"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"hosts"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"lastBackup"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"status"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BackupStatusFields"}}]}},{"kind":"Field","name":{"kind":"Name","value":"agentVersion"}}]}},{"kind":"Field","name":{"kind":"Name","value":"agentVersion"}},{"kind":"Field","name":{"kind":"Name","value":"availibilityState"}},{"kind":"Field","name":{"kind":"Name","value":"timeSinceLastBackup"}},{"kind":"Field","name":{"kind":"Name","value":"dateToNextBackup"}},{"kind":"Field","name":{"kind":"Name","value":"configuration"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"schedule"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"activated"}}]}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BackupStatusFields"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"BackupStatusDto"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"statusType"}},{"kind":"Field","name":{"kind":"Name","value":"finishingStage"}},{"kind":"Field","name":{"kind":"Name","value":"abortingStage"}},{"kind":"Field","name":{"kind":"Name","value":"failedStage"}},{"kind":"Field","name":{"kind":"Name","value":"removingStage"}}]}}]} as unknown as DocumentNode<HostsQuery, HostsQueryVariables>;
export const BackupDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"Backup"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"id"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"backup"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"hostname"},"value":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}}},{"kind":"Argument","name":{"kind":"Name","value":"id"},"value":{"kind":"Variable","name":{"kind":"Name","value":"id"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"status"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BackupStatusFields"}}]}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"endDate"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"existingFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"existingFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"shareRecords"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"path"}},{"kind":"Field","name":{"kind":"Name","value":"snapshotMethod"}},{"kind":"Field","name":{"kind":"Name","value":"snapshotFailureReason"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BackupStatusFields"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"BackupStatusDto"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"statusType"}},{"kind":"Field","name":{"kind":"Name","value":"finishingStage"}},{"kind":"Field","name":{"kind":"Name","value":"abortingStage"}},{"kind":"Field","name":{"kind":"Name","value":"failedStage"}},{"kind":"Field","name":{"kind":"Name","value":"removingStage"}}]}}]} as unknown as DocumentNode<BackupQuery, BackupQueryVariables>;
export const BackupsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"Backups"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"backups"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"hostname"},"value":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"status"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BackupStatusFields"}}]}},{"kind":"Field","name":{"kind":"Name","value":"retentionCategory"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"endDate"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"existingFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"existingFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BackupStatusFields"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"BackupStatusDto"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"statusType"}},{"kind":"Field","name":{"kind":"Name","value":"finishingStage"}},{"kind":"Field","name":{"kind":"Name","value":"abortingStage"}},{"kind":"Field","name":{"kind":"Name","value":"failedStage"}},{"kind":"Field","name":{"kind":"Name","value":"removingStage"}}]}}]} as unknown as DocumentNode<BackupsQuery, BackupsQueryVariables>;
export const BackupsBrowseDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"BackupsBrowse"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"id"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"sharePath"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"path"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Buffer"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"backup"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"hostname"},"value":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}}},{"kind":"Argument","name":{"kind":"Name","value":"id"},"value":{"kind":"Variable","name":{"kind":"Name","value":"id"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"files"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"sharePath"},"value":{"kind":"Variable","name":{"kind":"Name","value":"sharePath"}}},{"kind":"Argument","name":{"kind":"Name","value":"path"},"value":{"kind":"Variable","name":{"kind":"Name","value":"path"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"FragmentFileDescription"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"FragmentFileDescription"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"FileDescription"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"path"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"stats"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"ownerId"}},{"kind":"Field","name":{"kind":"Name","value":"groupId"}},{"kind":"Field","name":{"kind":"Name","value":"mode"}},{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"lastModified"}}]}},{"kind":"Field","name":{"kind":"Name","value":"symlink"}}]}}]} as unknown as DocumentNode<BackupsBrowseQuery, BackupsBrowseQueryVariables>;
export const CreateBackupDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"createBackup"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"createBackup"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"hostname"},"value":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]} as unknown as DocumentNode<CreateBackupMutation, CreateBackupMutationVariables>;
export const PurgeRetentionDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"purgeRetention"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"purgeRetention"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"hostname"},"value":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]} as unknown as DocumentNode<PurgeRetentionMutation, PurgeRetentionMutationVariables>;
export const RemoveBackupDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"removeBackup"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"id"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"removeBackup"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"hostname"},"value":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}}},{"kind":"Argument","name":{"kind":"Name","value":"id"},"value":{"kind":"Variable","name":{"kind":"Name","value":"id"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]} as unknown as DocumentNode<RemoveBackupMutation, RemoveBackupMutationVariables>;
export const SharesBrowseDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"SharesBrowse"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"id"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"backup"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"hostname"},"value":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}}},{"kind":"Argument","name":{"kind":"Name","value":"id"},"value":{"kind":"Variable","name":{"kind":"Name","value":"id"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"shares"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"FragmentFileDescription"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"FragmentFileDescription"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"FileDescription"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"path"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"stats"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"ownerId"}},{"kind":"Field","name":{"kind":"Name","value":"groupId"}},{"kind":"Field","name":{"kind":"Name","value":"mode"}},{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"lastModified"}}]}},{"kind":"Field","name":{"kind":"Name","value":"symlink"}}]}}]} as unknown as DocumentNode<SharesBrowseQuery, SharesBrowseQueryVariables>;
export const BackupUpdatedDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"subscription","name":{"kind":"Name","value":"BackupUpdated"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"backupUpdated"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"hostname"},"value":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"status"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"statusType"}},{"kind":"Field","name":{"kind":"Name","value":"finishingStage"}},{"kind":"Field","name":{"kind":"Name","value":"abortingStage"}},{"kind":"Field","name":{"kind":"Name","value":"failedStage"}},{"kind":"Field","name":{"kind":"Name","value":"removingStage"}}]}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"endDate"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"existingFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"existingFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}}]}}]}}]} as unknown as DocumentNode<BackupUpdatedSubscription, BackupUpdatedSubscriptionVariables>;
export const JobRemoveUpdatedDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"subscription","name":{"kind":"Name","value":"JobRemoveUpdated"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"host"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"kind"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"jobUpdated"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"host"},"value":{"kind":"Variable","name":{"kind":"Name","value":"host"}}},{"kind":"Argument","name":{"kind":"Name","value":"kind"},"value":{"kind":"Variable","name":{"kind":"Name","value":"kind"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"jobId"}},{"kind":"Field","name":{"kind":"Name","value":"status"}}]}}]}}]} as unknown as DocumentNode<JobRemoveUpdatedSubscription, JobRemoveUpdatedSubscriptionVariables>;
export const JobBackupUpdatedDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"subscription","name":{"kind":"Name","value":"JobBackupUpdated"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"host"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"kind"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"jobUpdated"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"host"},"value":{"kind":"Variable","name":{"kind":"Name","value":"host"}}},{"kind":"Argument","name":{"kind":"Name","value":"kind"},"value":{"kind":"Variable","name":{"kind":"Name","value":"kind"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"jobId"}},{"kind":"Field","name":{"kind":"Name","value":"status"}}]}}]}}]} as unknown as DocumentNode<JobBackupUpdatedSubscription, JobBackupUpdatedSubscriptionVariables>;
export const RestoreBackupDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"restoreBackup"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"RestoreInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"restoreBackup"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobPoolResponse"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobPoolResponse"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobResponse"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]} as unknown as DocumentNode<RestoreBackupMutation, RestoreBackupMutationVariables>;
export const ClearCacheDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"clearCache"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"clearCache"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]} as unknown as DocumentNode<ClearCacheMutation, ClearCacheMutationVariables>;
export const EventsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"Events"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"firstEvent"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"DateTime"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"lastEvent"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"DateTime"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"limit"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"offset"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"events"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"firstEvent"},"value":{"kind":"Variable","name":{"kind":"Name","value":"firstEvent"}}},{"kind":"Argument","name":{"kind":"Name","value":"lastEvent"},"value":{"kind":"Variable","name":{"kind":"Name","value":"lastEvent"}}},{"kind":"Argument","name":{"kind":"Name","value":"limit"},"value":{"kind":"Variable","name":{"kind":"Name","value":"limit"}}},{"kind":"Argument","name":{"kind":"Name","value":"offset"},"value":{"kind":"Variable","name":{"kind":"Name","value":"offset"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ApplicationEvent"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EventBackupInformation"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventBackupInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"hostname"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"sharePath"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EventPoolInformation"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventPoolInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"fix"}},{"kind":"Field","name":{"kind":"Name","value":"refcount"}},{"kind":"Field","name":{"kind":"Name","value":"refcountError"}},{"kind":"Field","name":{"kind":"Name","value":"inUnused"}},{"kind":"Field","name":{"kind":"Name","value":"inRefcnt"}},{"kind":"Field","name":{"kind":"Name","value":"inNothing"}},{"kind":"Field","name":{"kind":"Name","value":"missing"}},{"kind":"Field","name":{"kind":"Name","value":"chunkCount"}},{"kind":"Field","name":{"kind":"Name","value":"chunkError"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EventPoolCleanedInformation"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventPoolCleanedInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"count"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EventHashConversionInformation"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventHashConversionInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"count"}},{"kind":"Field","name":{"kind":"Name","value":"algorithm"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ApplicationEvent"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"ApplicationEvent"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"uuid"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"step"}},{"kind":"Field","name":{"kind":"Name","value":"source"}},{"kind":"Field","name":{"kind":"Name","value":"timestamp"}},{"kind":"Field","name":{"kind":"Name","value":"errorMessages"}},{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"information"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventBackupInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"EventBackupInformation"}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventPoolInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"EventPoolInformation"}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventPoolCleanedInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"EventPoolCleanedInformation"}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventHashConversionInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"EventHashConversionInformation"}}]}}]}}]}}]} as unknown as DocumentNode<EventsQuery, EventsQueryVariables>;
export const CleanupPoolDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"cleanupPool"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"cleanupPool"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobPoolResponse"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobPoolResponse"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobResponse"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]} as unknown as DocumentNode<CleanupPoolMutation, CleanupPoolMutationVariables>;
export const CheckAndFixPoolDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"checkAndFixPool"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"fix"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Boolean"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"verifyChunks"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Boolean"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"checkAndFixPool"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"fix"},"value":{"kind":"Variable","name":{"kind":"Name","value":"fix"}}},{"kind":"Argument","name":{"kind":"Name","value":"verifyChunks"},"value":{"kind":"Variable","name":{"kind":"Name","value":"verifyChunks"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobPoolResponse"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobPoolResponse"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobResponse"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]} as unknown as DocumentNode<CheckAndFixPoolMutation, CheckAndFixPoolMutationVariables>;
export const ServerInformationsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"ServerInformations"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"informations"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"uptime"}},{"kind":"Field","name":{"kind":"Name","value":"hostname"}},{"kind":"Field","name":{"kind":"Name","value":"woodstockVersion"}}]}}]}}]} as unknown as DocumentNode<ServerInformationsQuery, ServerInformationsQueryVariables>;
export const DiskUsageStatisticsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"DiskUsageStatistics"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"statistics"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"hosts"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"compressedSize"}}]}}]}}]}}]} as unknown as DocumentNode<DiskUsageStatisticsQuery, DiskUsageStatisticsQueryVariables>;
export const QueueStatisticsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"QueueStatistics"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"queueStats"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"pending"}},{"kind":"Field","name":{"kind":"Name","value":"running"}},{"kind":"Field","name":{"kind":"Name","value":"success"}},{"kind":"Field","name":{"kind":"Name","value":"failed"}},{"kind":"Field","name":{"kind":"Name","value":"dead"}}]}}]}}]} as unknown as DocumentNode<QueueStatisticsQuery, QueueStatisticsQueryVariables>;
export const PoolStatisticsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"PoolStatistics"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"statistics"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"diskUsage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"used"}},{"kind":"Field","name":{"kind":"Name","value":"usedLastMonth"}},{"kind":"Field","name":{"kind":"Name","value":"free"}},{"kind":"Field","name":{"kind":"Name","value":"total"}}]}},{"kind":"Field","name":{"kind":"Name","value":"poolUsage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"nbChunk"}},{"kind":"Field","name":{"kind":"Name","value":"nbChunkLastMonth"}},{"kind":"Field","name":{"kind":"Name","value":"nbChunkRange"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"time"}},{"kind":"Field","name":{"kind":"Name","value":"value"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nbRef"}},{"kind":"Field","name":{"kind":"Name","value":"nbRefLastMonth"}},{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"compressedSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedSizeLastMonth"}},{"kind":"Field","name":{"kind":"Name","value":"compressedSizeRange"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"time"}},{"kind":"Field","name":{"kind":"Name","value":"value"}}]}},{"kind":"Field","name":{"kind":"Name","value":"unusedSize"}}]}}]}}]}}]} as unknown as DocumentNode<PoolStatisticsQuery, PoolStatisticsQueryVariables>;
export const TasksDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"Tasks"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"QueueListInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"queue"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Job"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobBackupData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobBackupData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"ip"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobRestoreData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobRestoreData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"ip"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"destinationDirectory"}},{"kind":"Field","name":{"kind":"Name","value":"files"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"share"}},{"kind":"Field","name":{"kind":"Name","value":"selection"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobRemoveData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobRemoveData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobCleanupData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobCleanupData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"target"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobFsckData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobFsckData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"dryRun"}},{"kind":"Field","name":{"kind":"Name","value":"verifyChunks"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobArchiveData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobArchiveData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"profileName"}},{"kind":"Field","name":{"kind":"Name","value":"hostnames"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BackupTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobBackupTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"backupExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"backupErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"backupErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","alias":{"kind":"Name","value":"globalProgression"},"name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"startTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"endTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}}]}},{"kind":"Field","name":{"kind":"Name","value":"preCommandStates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"executionState"}},{"kind":"Field","name":{"kind":"Name","value":"command"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"command"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"shareStates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"share"}},{"kind":"Field","name":{"kind":"Name","value":"executionState"}},{"kind":"Field","name":{"kind":"Name","value":"backupProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"startTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"endTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}}]}},{"kind":"Field","name":{"kind":"Name","value":"fileListProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"postCommandStates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"executionState"}},{"kind":"Field","name":{"kind":"Name","value":"command"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"command"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RestoreTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobRestoreTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"restoreExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"restoreErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"restoreErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","alias":{"kind":"Name","value":"restoreProgression"},"name":{"kind":"Name","value":"globalProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"startTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"endTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RemoveTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobRemoveState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"removeExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"removeErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"removeErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"CleanerTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobCleanerTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"cleanerExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"cleanerErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"cleanerErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","alias":{"kind":"Name","value":"cleanerProgress"},"name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"FsckTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobFsckTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"fsckExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"fsckErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"fsckErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","name":{"kind":"Name","value":"dryRun"}},{"kind":"Field","name":{"kind":"Name","value":"refcntProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"totalCount"}}]}},{"kind":"Field","name":{"kind":"Name","value":"unusedProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"inNothing"}},{"kind":"Field","name":{"kind":"Name","value":"inRefcnt"}},{"kind":"Field","name":{"kind":"Name","value":"inUnused"}},{"kind":"Field","name":{"kind":"Name","value":"missing"}}]}},{"kind":"Field","name":{"kind":"Name","value":"chunkProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"totalCount"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ArchiveTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobArchiveTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"currentHost"}},{"kind":"Field","name":{"kind":"Name","value":"hostsDone"}},{"kind":"Field","name":{"kind":"Name","value":"hostsTotal"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"archiveSize"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"failedHosts"}},{"kind":"Field","name":{"kind":"Name","value":"hostStates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"hostname"}},{"kind":"Field","name":{"kind":"Name","value":"executionState"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"archiveSize"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Job"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Job"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"jobId"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"timestamp"}},{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"failedReason"}},{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobBackupData"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobRestoreData"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobRemoveData"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobCleanupData"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobFsckData"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobArchiveData"}}]}},{"kind":"Field","name":{"kind":"Name","value":"progress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BackupTaskState"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"RestoreTaskState"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"RemoveTaskState"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"CleanerTaskState"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"FsckTaskState"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"ArchiveTaskState"}}]}}]}}]} as unknown as DocumentNode<TasksQuery, TasksQueryVariables>;
export const QueueTasksJobUpdatedDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"subscription","name":{"kind":"Name","value":"QueueTasksJobUpdated"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"jobUpdated"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Job"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobBackupData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobBackupData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"ip"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobRestoreData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobRestoreData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"ip"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"destinationDirectory"}},{"kind":"Field","name":{"kind":"Name","value":"files"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"share"}},{"kind":"Field","name":{"kind":"Name","value":"selection"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobRemoveData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobRemoveData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobCleanupData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobCleanupData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"target"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobFsckData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobFsckData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"dryRun"}},{"kind":"Field","name":{"kind":"Name","value":"verifyChunks"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobArchiveData"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobArchiveData"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"profileName"}},{"kind":"Field","name":{"kind":"Name","value":"hostnames"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BackupTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobBackupTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"backupExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"backupErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"backupErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","alias":{"kind":"Name","value":"globalProgression"},"name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"startTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"endTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}}]}},{"kind":"Field","name":{"kind":"Name","value":"preCommandStates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"executionState"}},{"kind":"Field","name":{"kind":"Name","value":"command"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"command"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"shareStates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"share"}},{"kind":"Field","name":{"kind":"Name","value":"executionState"}},{"kind":"Field","name":{"kind":"Name","value":"backupProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"startTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"endTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}}]}},{"kind":"Field","name":{"kind":"Name","value":"fileListProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"postCommandStates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"executionState"}},{"kind":"Field","name":{"kind":"Name","value":"command"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"command"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RestoreTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobRestoreTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"restoreExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"restoreErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"restoreErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","alias":{"kind":"Name","value":"restoreProgression"},"name":{"kind":"Name","value":"globalProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"startTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"endTransferDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RemoveTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobRemoveState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"removeExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"removeErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"removeErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"CleanerTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobCleanerTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"cleanerExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"cleanerErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"cleanerErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","alias":{"kind":"Name","value":"cleanerProgress"},"name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"FsckTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobFsckTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","alias":{"kind":"Name","value":"fsckExecutionState"},"name":{"kind":"Name","value":"executionState"}},{"kind":"Field","alias":{"kind":"Name","value":"fsckErrorState"},"name":{"kind":"Name","value":"errorState"}},{"kind":"Field","alias":{"kind":"Name","value":"fsckErrorMessage"},"name":{"kind":"Name","value":"errorMessage"}},{"kind":"Field","name":{"kind":"Name","value":"dryRun"}},{"kind":"Field","name":{"kind":"Name","value":"refcntProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"totalCount"}}]}},{"kind":"Field","name":{"kind":"Name","value":"unusedProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"inNothing"}},{"kind":"Field","name":{"kind":"Name","value":"inRefcnt"}},{"kind":"Field","name":{"kind":"Name","value":"inUnused"}},{"kind":"Field","name":{"kind":"Name","value":"missing"}}]}},{"kind":"Field","name":{"kind":"Name","value":"chunkProgression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"totalCount"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ArchiveTaskState"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobArchiveTaskState"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"currentHost"}},{"kind":"Field","name":{"kind":"Name","value":"hostsDone"}},{"kind":"Field","name":{"kind":"Name","value":"hostsTotal"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"archiveSize"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}},{"kind":"Field","name":{"kind":"Name","value":"failedHosts"}},{"kind":"Field","name":{"kind":"Name","value":"hostStates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"hostname"}},{"kind":"Field","name":{"kind":"Name","value":"executionState"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"archiveSize"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Job"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Job"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"jobId"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"timestamp"}},{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"failedReason"}},{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobBackupData"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobRestoreData"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobRemoveData"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobCleanupData"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobFsckData"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobArchiveData"}}]}},{"kind":"Field","name":{"kind":"Name","value":"progress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BackupTaskState"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"RestoreTaskState"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"RemoveTaskState"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"CleanerTaskState"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"FsckTaskState"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"ArchiveTaskState"}}]}}]}}]} as unknown as DocumentNode<QueueTasksJobUpdatedSubscription, QueueTasksJobUpdatedSubscriptionVariables>;
import { dateTypePolicy, bigintTypePolicy } from '../utils/graphql.utils';

export const scalarTypePolicies = {
  ApplicationEvent: { fields: { timestamp: dateTypePolicy } },
  ArchiveHostState: {
    fields: { archiveSize: bigintTypePolicy, progressCurrent: bigintTypePolicy, progressMax: bigintTypePolicy },
  },
  BackupEx: {
    fields: {
      compressedFileSize: bigintTypePolicy,
      endDate: dateTypePolicy,
      existingCompressedFileSize: bigintTypePolicy,
      existingFileSize: bigintTypePolicy,
      fileSize: bigintTypePolicy,
      modifiedCompressedFileSize: bigintTypePolicy,
      modifiedFileSize: bigintTypePolicy,
      newCompressedFileSize: bigintTypePolicy,
      newFileSize: bigintTypePolicy,
      startDate: dateTypePolicy,
    },
  },
  BackupProgression: {
    fields: {
      compressedFileSize: bigintTypePolicy,
      endTransferDate: dateTypePolicy,
      fileSize: bigintTypePolicy,
      modifiedCompressedFileSize: bigintTypePolicy,
      modifiedFileSize: bigintTypePolicy,
      newCompressedFileSize: bigintTypePolicy,
      newFileSize: bigintTypePolicy,
      progressCurrent: bigintTypePolicy,
      progressMax: bigintTypePolicy,
      startDate: dateTypePolicy,
      startTransferDate: dateTypePolicy,
    },
  },
  BigIntTimeSerie: { fields: { time: dateTypePolicy, value: bigintTypePolicy } },
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
    fields: {
      compressedSize: bigintTypePolicy,
      dev: bigintTypePolicy,
      ino: bigintTypePolicy,
      nlink: bigintTypePolicy,
      rdev: bigintTypePolicy,
      size: bigintTypePolicy,
    },
  },
  Host: { fields: { dateToNextBackup: dateTypePolicy } },
  HostStatistics: {
    fields: {
      compressedSize: bigintTypePolicy,
      compressedSizeLastMonth: bigintTypePolicy,
      size: bigintTypePolicy,
      sizeLastMonth: bigintTypePolicy,
    },
  },
  JobArchiveTaskState: {
    fields: { archiveSize: bigintTypePolicy, progressCurrent: bigintTypePolicy, progressMax: bigintTypePolicy },
  },
  JobBackupData: { fields: { startDate: dateTypePolicy } },
  JobRemoveData: { fields: { startDate: dateTypePolicy } },
  JobRestoreData: { fields: { startDate: dateTypePolicy } },
  NumberTimeSerie: { fields: { time: dateTypePolicy } },
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
  QueueStats: { fields: { lastExecution: dateTypePolicy, nextWakeup: dateTypePolicy } },
};
