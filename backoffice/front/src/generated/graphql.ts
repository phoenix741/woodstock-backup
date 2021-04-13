export type Maybe<T> = T | null;
export type Exact<T extends { [key: string]: any }> = { [K in keyof T]: T[K] };
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: string;
  String: string;
  Boolean: boolean;
  Int: number;
  Float: number;
  /** `Date` type as integer. Type represents date and time as number of milliseconds from start of UNIX epoch. */
  Timestamp: number;
};

export type FileDescription = {
  name: Scalars['String'];
  type: EnumFileType;
  dev: Scalars['Float'];
  ino: Scalars['Float'];
  mode: Scalars['Float'];
  nlink: Scalars['Float'];
  uid: Scalars['Float'];
  gid: Scalars['Float'];
  rdev: Scalars['Float'];
  size: Scalars['Float'];
  blksize: Scalars['Float'];
  blocks: Scalars['Float'];
  atimeMs: Scalars['Float'];
  mtimeMs: Scalars['Float'];
  ctimeMs: Scalars['Float'];
  birthtimeMs: Scalars['Float'];
  atime: Scalars['Timestamp'];
  mtime: Scalars['Timestamp'];
  ctime: Scalars['Timestamp'];
  birthtime: Scalars['Timestamp'];
};

export enum EnumFileType {
  BlockDevice = 'BLOCK_DEVICE',
  CharacterDevice = 'CHARACTER_DEVICE',
  Directory = 'DIRECTORY',
  Fifo = 'FIFO',
  RegularFile = 'REGULAR_FILE',
  Socket = 'SOCKET',
  SymbolicLink = 'SYMBOLIC_LINK',
  Unknown = 'UNKNOWN'
}


export type ScheduledBackupToKeep = {
  hourly: Scalars['Float'];
  daily: Scalars['Float'];
  weekly: Scalars['Float'];
  monthly: Scalars['Float'];
  yearly: Scalars['Float'];
};

export type Schedule = {
  activated: Scalars['Boolean'];
  backupPerdiod: Scalars['Float'];
  backupToKeep: ScheduledBackupToKeep;
};

export type BackupTaskShare = {
  checksum?: Maybe<Scalars['Boolean']>;
  name: Scalars['String'];
  includes?: Maybe<Array<Scalars['String']>>;
  excludes?: Maybe<Array<Scalars['String']>>;
};

export type DhcpAddress = {
  address: Scalars['String'];
  start: Scalars['Float'];
  end: Scalars['Float'];
};

export type HostConfigOperation = {
  tasks?: Maybe<Array<Operation>>;
  finalizeTasks?: Maybe<Array<Operation>>;
};

export type Operation = ExecuteCommandOperation | RSyncBackupOperation | RSyncdBackupOperation;

export type ExecuteCommandOperation = {
  name: Scalars['String'];
  command: Scalars['String'];
};

export type RSyncBackupOperation = {
  name: Scalars['String'];
  share: Array<BackupTaskShare>;
  includes?: Maybe<Array<Scalars['String']>>;
  excludes?: Maybe<Array<Scalars['String']>>;
  timeout?: Maybe<Scalars['Float']>;
};

export type RSyncdBackupOperation = {
  name: Scalars['String'];
  authentification?: Maybe<Scalars['Boolean']>;
  username?: Maybe<Scalars['String']>;
  password?: Maybe<Scalars['String']>;
  share: Array<BackupTaskShare>;
  includes?: Maybe<Array<Scalars['String']>>;
  excludes?: Maybe<Array<Scalars['String']>>;
  timeout?: Maybe<Scalars['Float']>;
};

export type HostConfiguration = {
  addresses?: Maybe<Array<Scalars['String']>>;
  dhcp?: Maybe<Array<DhcpAddress>>;
  operations?: Maybe<HostConfigOperation>;
  schedule?: Maybe<Schedule>;
};

export type DiskUsageStatisticsRecord = {
  diskUsage: Scalars['Float'];
  uncompressed: Scalars['Float'];
};

export type DiskUsageStatistics = {
  total?: Maybe<DiskUsageStatisticsRecord>;
  none?: Maybe<DiskUsageStatisticsRecord>;
  zlib?: Maybe<DiskUsageStatisticsRecord>;
  lzo?: Maybe<DiskUsageStatisticsRecord>;
  zstd?: Maybe<DiskUsageStatisticsRecord>;
};

export type Backup = {
  number: Scalars['Float'];
  complete: Scalars['Boolean'];
  startDate: Scalars['Float'];
  endDate?: Maybe<Scalars['Float']>;
  fileCount: Scalars['Float'];
  newFileCount: Scalars['Float'];
  existingFileCount: Scalars['Float'];
  fileSize: Scalars['Float'];
  existingFileSize: Scalars['Float'];
  newFileSize: Scalars['Float'];
  speed: Scalars['Float'];
  diskUsageStatistics?: Maybe<DiskUsageStatistics>;
  shares: Array<FileDescription>;
  files: Array<FileDescription>;
};


export type BackupFilesArgs = {
  path: Scalars['String'];
  sharePath: Scalars['String'];
};

export type JobResponse = {
  id: Scalars['Float'];
};

export type Host = {
  name: Scalars['String'];
  configuration: HostConfiguration;
  backups: Array<Backup>;
  lastBackup?: Maybe<Backup>;
  lastBackupState?: Maybe<Scalars['String']>;
};

export type TaskProgression = {
  fileSize: Scalars['Float'];
  newFileSize: Scalars['Float'];
  newFileCount: Scalars['Float'];
  fileCount: Scalars['Float'];
  speed: Scalars['Float'];
  percent: Scalars['Float'];
};

export type BackupSubTask = {
  context: Scalars['String'];
  description: Scalars['String'];
  state: BackupState;
  progression?: Maybe<TaskProgression>;
};

export enum BackupState {
  Waiting = 'WAITING',
  Running = 'RUNNING',
  Success = 'SUCCESS',
  Aborted = 'ABORTED',
  Failed = 'FAILED'
}

export type BackupTask = {
  complete?: Maybe<Scalars['Boolean']>;
  host: Scalars['String'];
  config?: Maybe<HostConfiguration>;
  previousNumber?: Maybe<Scalars['Float']>;
  number?: Maybe<Scalars['Float']>;
  ip?: Maybe<Scalars['String']>;
  startDate?: Maybe<Scalars['Float']>;
  subtasks?: Maybe<Array<BackupSubTask>>;
  state?: Maybe<BackupState>;
  progression?: Maybe<TaskProgression>;
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

export type QueueStats = {
  waiting: Scalars['Int'];
  active: Scalars['Int'];
  failed: Scalars['Int'];
  delayed: Scalars['Int'];
  completed: Scalars['Int'];
  lastExecution: Scalars['Float'];
  nextWakeup: Scalars['Float'];
};

export type BtrfsCheckTools = {
  btrfstools?: Maybe<Scalars['Boolean']>;
  compsize?: Maybe<Scalars['Boolean']>;
};

export type BtrfsCheck = {
  isBtrfsVolume?: Maybe<Scalars['Boolean']>;
  hasAuthorization?: Maybe<Scalars['Boolean']>;
  backupVolume?: Maybe<Scalars['String']>;
  backupVolumeFileSystem?: Maybe<Scalars['String']>;
  toolsAvailable: BtrfsCheckTools;
};

export type CompressionStatistics = {
  timestamp: Scalars['Float'];
  diskUsage: Scalars['Float'];
  uncompressed: Scalars['Float'];
};

export type SpaceStatistics = {
  size: Scalars['Float'];
  used: Scalars['Float'];
  free: Scalars['Float'];
};

export type BackupQuota = {
  host: Scalars['String'];
  number: Scalars['Float'];
  refr: Scalars['Float'];
  excl: Scalars['Float'];
  total: Scalars['Float'];
};

export type HostQuota = {
  host: Scalars['String'];
  excl: Scalars['Float'];
  refr: Scalars['Float'];
  total: Scalars['Float'];
};

export type TotalQuota = {
  refr: Scalars['Float'];
  excl: Scalars['Float'];
  total: Scalars['Float'];
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

export type DiskUsageStats = {
  quotas: Array<TimestampBackupQuota>;
  currentSpace: SpaceStatistics;
  currentRepartition?: Maybe<Array<HostQuota>>;
  compressionStats: Array<CompressionStatistics>;
};

export type Query = {
  backups: Array<Backup>;
  backup: Backup;
  hosts: Array<Host>;
  host: Host;
  queue: Array<Job>;
  queueStats: QueueStats;
  status: BtrfsCheck;
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

export type Subscription = {
  jobUpdated: Job;
  jobWaiting: Scalars['Int'];
  jobFailed: Job;
  jobRemoved: Job;
};

export type NavigationBarTasksQueryVariables = Exact<{
  state?: Maybe<Array<Scalars['String']>>;
}>;


export type NavigationBarTasksQuery = { queue: Array<Pick<Job, 'id' | 'state'>> };

export type NavigationBarTasksJobUpdatedSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type NavigationBarTasksJobUpdatedSubscription = { jobUpdated: Pick<Job, 'id' | 'state'> };

export type RunningTasksMenuQueryVariables = Exact<{
  state?: Maybe<Array<Scalars['String']>>;
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


export type BackupsBrowseQuery = { backup: { files: Array<Pick<FileDescription, 'name' | 'type' | 'uid' | 'gid' | 'mode' | 'size' | 'mtime'>> } };

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
  state?: Maybe<Array<Scalars['String']>>;
}>;


export type QueueTasksQuery = { queue: Array<FragmentJobFragment> };

export type QueueTasksJobUpdatedSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type QueueTasksJobUpdatedSubscription = { jobUpdated: FragmentJobFragment };

export type SharesBrowseQueryVariables = Exact<{
  hostname: Scalars['String'];
  number: Scalars['Int'];
}>;


export type SharesBrowseQuery = { backup: { shares: Array<Pick<FileDescription, 'name' | 'type' | 'uid' | 'gid' | 'mode' | 'size' | 'mtime'>> } };
