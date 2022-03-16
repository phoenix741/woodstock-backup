export type Maybe<T> = T | null;
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
  /** The `BigInt` scalar type represents non-fractional signed whole numeric values.BigInt can represent values between -(2^63) + 1 and 2^63 - 1. */
  BigInt: any;
};

export type Backup = {
  fileSize: Scalars['BigInt'];
  existingFileSize: Scalars['BigInt'];
  newFileSize: Scalars['BigInt'];
  compressedFileSize: Scalars['BigInt'];
  existingCompressedFileSize: Scalars['BigInt'];
  newCompressedFileSize: Scalars['BigInt'];
  number: Scalars['Float'];
  complete: Scalars['Boolean'];
  startDate: Scalars['Float'];
  endDate?: Maybe<Scalars['Float']>;
  fileCount: Scalars['Float'];
  newFileCount: Scalars['Float'];
  existingFileCount: Scalars['Float'];
  speed: Scalars['Float'];
  shares: Array<FileDescription>;
  files: Array<FileDescription>;
};


export type BackupFilesArgs = {
  path: Scalars['String'];
  sharePath: Scalars['String'];
};

export type BackupOperation = {
  name: Scalars['String'];
  shares: Array<BackupTaskShare>;
  includes?: Maybe<Array<Scalars['String']>>;
  excludes?: Maybe<Array<Scalars['String']>>;
  timeout?: Maybe<Scalars['Float']>;
};

export type BackupQuota = {
  host: Scalars['String'];
  number: Scalars['Float'];
  refr: Scalars['Float'];
  excl: Scalars['Float'];
  total: Scalars['Float'];
};

export enum BackupState {
  Waiting = 'WAITING',
  Running = 'RUNNING',
  Success = 'SUCCESS',
  Aborted = 'ABORTED',
  Failed = 'FAILED'
}

export type BackupSubTask = {
  context: Scalars['String'];
  description: Scalars['String'];
  state: BackupState;
  progression?: Maybe<TaskProgression>;
};

export type BackupTask = {
  complete?: Maybe<Scalars['Boolean']>;
  host: Scalars['String'];
  config?: Maybe<HostConfiguration>;
  previousNumber?: Maybe<Scalars['Float']>;
  number?: Maybe<Scalars['Float']>;
  ip?: Maybe<Scalars['String']>;
  startDate?: Maybe<Scalars['Float']>;
  originalStartDate?: Maybe<Scalars['Float']>;
  subtasks?: Maybe<Array<BackupSubTask>>;
  state?: Maybe<BackupState>;
  progression?: Maybe<TaskProgression>;
};

export type BackupTaskShare = {
  name: Scalars['String'];
  includes?: Maybe<Array<Scalars['String']>>;
  excludes?: Maybe<Array<Scalars['String']>>;
};


export type CompressionStatistics = {
  timestamp: Scalars['Float'];
  diskUsage: Scalars['Float'];
  uncompressed: Scalars['Float'];
};

export type DhcpAddress = {
  address: Scalars['String'];
  start: Scalars['Float'];
  end: Scalars['Float'];
};

export type DiskUsageStats = {
  quotas: Array<TimestampBackupQuota>;
  currentSpace: SpaceStatistics;
  currentRepartition?: Maybe<Array<HostQuota>>;
  compressionStats: Array<CompressionStatistics>;
};

export enum EnumFileType {
  Share = 'SHARE',
  BlockDevice = 'BLOCK_DEVICE',
  CharacterDevice = 'CHARACTER_DEVICE',
  Directory = 'DIRECTORY',
  Fifo = 'FIFO',
  RegularFile = 'REGULAR_FILE',
  Socket = 'SOCKET',
  SymbolicLink = 'SYMBOLIC_LINK',
  Unknown = 'UNKNOWN'
}

export type ExecuteCommandOperation = {
  name: Scalars['String'];
  command: Scalars['String'];
};

export type FileAcl = {
  user?: Maybe<Scalars['String']>;
  group?: Maybe<Scalars['String']>;
  mask?: Maybe<Scalars['Float']>;
  other?: Maybe<Scalars['Float']>;
};

export type FileDescription = {
  path: Scalars['String'];
  symlink?: Maybe<Scalars['String']>;
  type: EnumFileType;
  stats: FileStat;
  acl: Array<FileAcl>;
};

export type FileStat = {
  ownerId?: Maybe<Scalars['String']>;
  groupId?: Maybe<Scalars['String']>;
  size?: Maybe<Scalars['String']>;
  compressedSize?: Maybe<Scalars['String']>;
  lastRead?: Maybe<Scalars['String']>;
  lastModified?: Maybe<Scalars['String']>;
  created?: Maybe<Scalars['String']>;
  mode?: Maybe<Scalars['String']>;
  dev?: Maybe<Scalars['String']>;
  rdev?: Maybe<Scalars['String']>;
  ino?: Maybe<Scalars['String']>;
  nlink?: Maybe<Scalars['String']>;
};

export type Host = {
  name: Scalars['String'];
  configuration: HostConfiguration;
  backups: Array<Backup>;
  lastBackup?: Maybe<Backup>;
  lastBackupState?: Maybe<Scalars['String']>;
};

export type HostConfigOperation = {
  tasks?: Maybe<Array<Operation>>;
  finalizeTasks?: Maybe<Array<Operation>>;
};

export type HostConfiguration = {
  isLocal?: Maybe<Scalars['Boolean']>;
  addresses?: Maybe<Array<Scalars['String']>>;
  dhcp?: Maybe<Array<DhcpAddress>>;
  operations?: Maybe<HostConfigOperation>;
  schedule?: Maybe<Schedule>;
};

export type HostQuota = {
  host: Scalars['String'];
  excl: Scalars['Float'];
  refr: Scalars['Float'];
  total: Scalars['Float'];
};

export type Job = {
  id: Scalars['Int'];
  delay: Scalars['Int'];
  timestamp: Scalars['Int'];
  attemptsMade: Scalars['Int'];
  finishedOn?: Maybe<Scalars['Int']>;
  processedOn?: Maybe<Scalars['Int']>;
  name: Scalars['String'];
  data: BackupTask;
  failedReason?: Maybe<Scalars['String']>;
  stacktrace?: Maybe<Array<Scalars['String']>>;
  state?: Maybe<Scalars['String']>;
  progress?: Maybe<Scalars['Float']>;
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
  number: Scalars['Int'];
  hostname: Scalars['String'];
};

export type Operation = ExecuteCommandOperation | BackupOperation;

export type Query = {
  backups: Array<Backup>;
  backup: Backup;
  hosts: Array<Host>;
  host: Host;
  queue: Array<Job>;
  queueStats: QueueStats;
  diskUsageStats: DiskUsageStats;
};


export type QueryBackupsArgs = {
  hostname: Scalars['String'];
};


export type QueryBackupArgs = {
  number: Scalars['Int'];
  hostname: Scalars['String'];
};


export type QueryHostArgs = {
  hostname: Scalars['String'];
};


export type QueryQueueArgs = {
  state?: Maybe<Array<Scalars['String']>>;
};

export type QueueStats = {
  waiting: Scalars['Int'];
  active: Scalars['Int'];
  failed: Scalars['Int'];
  delayed: Scalars['Int'];
  completed: Scalars['Int'];
  lastExecution: Scalars['Float'];
  nextWakeup: Scalars['Float'];
};

export type Schedule = {
  activated?: Maybe<Scalars['Boolean']>;
  backupPerdiod?: Maybe<Scalars['Float']>;
  backupToKeep?: Maybe<ScheduledBackupToKeep>;
};

export type ScheduledBackupToKeep = {
  hourly?: Maybe<Scalars['Float']>;
  daily?: Maybe<Scalars['Float']>;
  weekly?: Maybe<Scalars['Float']>;
  monthly?: Maybe<Scalars['Float']>;
  yearly?: Maybe<Scalars['Float']>;
};

export type SpaceStatistics = {
  size: Scalars['Float'];
  used: Scalars['Float'];
  free: Scalars['Float'];
};

export type Subscription = {
  jobUpdated: Job;
  jobWaiting: Scalars['Int'];
  jobFailed: Job;
  jobRemoved: Job;
};

export type TaskProgression = {
  compressedFileSize: Scalars['BigInt'];
  newCompressedFileSize: Scalars['BigInt'];
  fileSize: Scalars['BigInt'];
  newFileSize: Scalars['BigInt'];
  percent: Scalars['Int'];
  progressCurrent: Scalars['BigInt'];
  progressMax: Scalars['BigInt'];
  newFileCount: Scalars['Float'];
  fileCount: Scalars['Float'];
  speed: Scalars['Float'];
};

export type TimestampBackupQuota = {
  timestamp: Scalars['Float'];
  volumes: Array<BackupQuota>;
  space: SpaceStatistics;
  host: Array<HostQuota>;
  total: TotalQuota;
};


export type TimestampBackupQuotaVolumesArgs = {
  host?: Maybe<Scalars['String']>;
};


export type TimestampBackupQuotaHostArgs = {
  host?: Maybe<Scalars['String']>;
};

export type TotalQuota = {
  refr: Scalars['Float'];
  excl: Scalars['Float'];
  total: Scalars['Float'];
};

export type NavigationBarTasksQueryVariables = Exact<{
  state?: Maybe<Array<Scalars['String']> | Scalars['String']>;
}>;


export type NavigationBarTasksQuery = { queue: Array<Pick<Job, 'id' | 'state'>> };

export type NavigationBarTasksJobUpdatedSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type NavigationBarTasksJobUpdatedSubscription = { jobUpdated: Pick<Job, 'id' | 'state'> };

export type RunningTasksMenuQueryVariables = Exact<{
  state?: Maybe<Array<Scalars['String']> | Scalars['String']>;
}>;


export type RunningTasksMenuQuery = { queue: Array<(
    Pick<Job, 'id' | 'progress' | 'state'>
    & { data: (
      Pick<BackupTask, 'host'>
      & { progression?: Maybe<Pick<TaskProgression, 'fileCount'>> }
    ) }
  )> };

export type RunningTasksMenuJobUpdatedSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type RunningTasksMenuJobUpdatedSubscription = { jobUpdated: (
    Pick<Job, 'id' | 'progress' | 'state'>
    & { data: (
      Pick<BackupTask, 'host'>
      & { progression?: Maybe<Pick<TaskProgression, 'fileCount'>> }
    ) }
  ) };

export type BackupsQueryVariables = Exact<{
  hostname: Scalars['String'];
}>;


export type BackupsQuery = { backups: Array<Pick<Backup, 'number' | 'complete' | 'startDate' | 'endDate' | 'fileCount' | 'newFileCount' | 'existingFileCount' | 'fileSize' | 'newFileSize' | 'existingFileSize' | 'speed'>> };

export type BackupsBrowseQueryVariables = Exact<{
  hostname: Scalars['String'];
  number: Scalars['Int'];
  sharePath: Scalars['String'];
  path: Scalars['String'];
}>;


export type BackupsBrowseQuery = { backup: { files: Array<FragmentFileDescriptionFragment> } };

export type CreateBackupMutationVariables = Exact<{
  hostname: Scalars['String'];
}>;


export type CreateBackupMutation = { createBackup: Pick<JobResponse, 'id'> };

export type RemoveBackupMutationVariables = Exact<{
  hostname: Scalars['String'];
  number: Scalars['Int'];
}>;


export type RemoveBackupMutation = { removeBackup: Pick<JobResponse, 'id'> };

export type DashboardQueryVariables = Exact<{ [key: string]: never; }>;


export type DashboardQuery = { queueStats: Pick<QueueStats, 'waiting' | 'active' | 'failed' | 'lastExecution' | 'nextWakeup'>, diskUsageStats: { currentRepartition?: Maybe<Array<Pick<HostQuota, 'host' | 'total'>>>, compressionStats: Array<Pick<CompressionStatistics, 'timestamp' | 'diskUsage' | 'uncompressed'>>, currentSpace: Pick<SpaceStatistics, 'size' | 'used'>, quotas: Array<(
      Pick<TimestampBackupQuota, 'timestamp'>
      & { total: Pick<TotalQuota, 'refr' | 'excl' | 'total'> }
    )> } };

export type FragmentFileDescriptionFragment = (
  Pick<FileDescription, 'path' | 'type' | 'symlink'>
  & { stats: Pick<FileStat, 'ownerId' | 'groupId' | 'mode' | 'size' | 'lastModified'> }
);

export type FragmentJobFragment = (
  Pick<Job, 'id' | 'state' | 'failedReason'>
  & { data: (
    Pick<BackupTask, 'host' | 'number' | 'startDate' | 'state'>
    & { progression?: Maybe<Pick<TaskProgression, 'percent' | 'speed' | 'newFileCount' | 'fileCount'>>, subtasks?: Maybe<Array<Pick<BackupSubTask, 'context' | 'description' | 'state'>>> }
  ) }
);

export type HostsQueryVariables = Exact<{ [key: string]: never; }>;


export type HostsQuery = { hosts: Array<(
    Pick<Host, 'name' | 'lastBackupState'>
    & { lastBackup?: Maybe<Pick<Backup, 'number' | 'startDate' | 'fileSize' | 'complete'>>, configuration: { schedule?: Maybe<Pick<Schedule, 'activated'>> } }
  )> };

export type QueueTasksQueryVariables = Exact<{
  state?: Maybe<Array<Scalars['String']> | Scalars['String']>;
}>;


export type QueueTasksQuery = { queue: Array<FragmentJobFragment> };

export type QueueTasksJobUpdatedSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type QueueTasksJobUpdatedSubscription = { jobUpdated: FragmentJobFragment };

export type SharesBrowseQueryVariables = Exact<{
  hostname: Scalars['String'];
  number: Scalars['Int'];
}>;


export type SharesBrowseQuery = { backup: { shares: Array<FragmentFileDescriptionFragment> } };
