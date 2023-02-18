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
  /** The `BigInt` scalar type represents non-fractional signed whole numeric values. BigInt can represent values between -(2^63) + 1 and 2^63 - 1. */
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
  sharePath: Scalars['String'];
  path: Scalars['String'];
};

export type BackupOperation = {
  shares: Array<BackupTaskShare>;
  includes?: Maybe<Array<Scalars['String']>>;
  excludes?: Maybe<Array<Scalars['String']>>;
  timeout?: Maybe<Scalars['Float']>;
};

export type BackupTask = {
  subtasks: Array<SubTaskOrGroupTasks>;
  groupName?: Maybe<Scalars['String']>;
  state: QueueTaskState;
  progression?: Maybe<JobProgression>;
  description?: Maybe<Scalars['String']>;
  host: Scalars['String'];
  number?: Maybe<Scalars['Float']>;
  ip?: Maybe<Scalars['String']>;
  startDate?: Maybe<Scalars['Float']>;
};

/**
 * Part of config file.
 *
 * Store information about a share
 */
export type BackupTaskShare = {
  name: Scalars['String'];
  includes?: Maybe<Array<Scalars['String']>>;
  excludes?: Maybe<Array<Scalars['String']>>;
};


/**
 * Part of config file
 *
 * Store information about a DHCP Address
 */
export type DhcpAddress = {
  address: Scalars['String'];
  start: Scalars['Float'];
  end: Scalars['Float'];
};

export type DiskUsage = {
  used?: Maybe<Scalars['Int']>;
  free?: Maybe<Scalars['Int']>;
  total?: Maybe<Scalars['Int']>;
  usedRange?: Maybe<Array<TimeSerie>>;
  usedLastMonth?: Maybe<Scalars['Float']>;
  freeRange?: Maybe<Array<TimeSerie>>;
  freeLastMonth?: Maybe<Scalars['Float']>;
  totalRange?: Maybe<Array<TimeSerie>>;
  totalLastMonth?: Maybe<Scalars['Float']>;
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
  xattr: Scalars['String'];
  symlink?: Maybe<Scalars['String']>;
  type: EnumFileType;
  stats?: Maybe<FileStat>;
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
  preCommands?: Maybe<Array<ExecuteCommandOperation>>;
  operation?: Maybe<BackupOperation>;
  postCommands?: Maybe<Array<ExecuteCommandOperation>>;
};

/**
 * Config file for one Host
 *
 * Contains all information that can be used to backup a host.
 */
export type HostConfiguration = {
  isLocal?: Maybe<Scalars['Boolean']>;
  password: Scalars['String'];
  addresses?: Maybe<Array<Scalars['String']>>;
  dhcp?: Maybe<Array<DhcpAddress>>;
  operations?: Maybe<HostConfigOperation>;
  schedule?: Maybe<Schedule>;
};

export type HostStatistics = {
  longestChain?: Maybe<Scalars['Int']>;
  nbChunk?: Maybe<Scalars['Int']>;
  nbRef?: Maybe<Scalars['Int']>;
  size?: Maybe<Scalars['Int']>;
  compressedSize?: Maybe<Scalars['Int']>;
  host?: Maybe<Scalars['String']>;
  longestChainRange?: Maybe<Array<TimeSerie>>;
  longestChainLastMonth?: Maybe<Scalars['Float']>;
  nbChunkRange?: Maybe<Array<TimeSerie>>;
  nbChunkLastMonth?: Maybe<Scalars['Float']>;
  nbRefRange?: Maybe<Array<TimeSerie>>;
  nbRefLastMonth?: Maybe<Scalars['Float']>;
  sizeRange?: Maybe<Array<TimeSerie>>;
  sizeLastMonth?: Maybe<Scalars['Float']>;
  compressedSizeRange?: Maybe<Array<TimeSerie>>;
  compressedSizeLastMonth?: Maybe<Scalars['Float']>;
};

export type Job = {
  attemptsMade: Scalars['Int'];
  id?: Maybe<Scalars['String']>;
  name: Scalars['String'];
  state: Scalars['String'];
  data: BackupTask;
};

export type JobGroupTasks = {
  subtasks: Array<SubTaskOrGroupTasks>;
  groupName?: Maybe<Scalars['String']>;
  state: QueueTaskState;
  progression?: Maybe<JobProgression>;
  description?: Maybe<Scalars['String']>;
};

export type JobProgression = {
  compressedFileSize: Scalars['BigInt'];
  newCompressedFileSize: Scalars['BigInt'];
  fileSize: Scalars['BigInt'];
  newFileSize: Scalars['BigInt'];
  newFileCount: Scalars['Int'];
  fileCount: Scalars['Int'];
  progressCurrent: Scalars['BigInt'];
  progressMax: Scalars['BigInt'];
  speed: Scalars['Float'];
  percent: Scalars['Float'];
};

export type JobResponse = {
  id: Scalars['String'];
};

export type JobSubTask = {
  taskName: Scalars['String'];
  state: QueueTaskState;
  progression?: Maybe<JobProgression>;
  description?: Maybe<Scalars['String']>;
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

export type PoolUsage = {
  longestChain?: Maybe<Scalars['Int']>;
  nbChunk?: Maybe<Scalars['Int']>;
  nbRef?: Maybe<Scalars['Int']>;
  size?: Maybe<Scalars['Int']>;
  compressedSize?: Maybe<Scalars['Int']>;
  longestChainRange?: Maybe<Array<TimeSerie>>;
  longestChainLastMonth?: Maybe<Scalars['Float']>;
  nbChunkRange?: Maybe<Array<TimeSerie>>;
  nbChunkLastMonth?: Maybe<Scalars['Float']>;
  nbRefRange?: Maybe<Array<TimeSerie>>;
  nbRefLastMonth?: Maybe<Scalars['Float']>;
  sizeRange?: Maybe<Array<TimeSerie>>;
  sizeLastMonth?: Maybe<Scalars['Float']>;
  compressedSizeRange?: Maybe<Array<TimeSerie>>;
  compressedSizeLastMonth?: Maybe<Scalars['Float']>;
};

export type Query = {
  backups: Array<Backup>;
  backup: Backup;
  hosts: Array<Host>;
  host: Host;
  queue: Array<Job>;
  queueStats: QueueStats;
  statistics: Statistics;
};


export type QueryBackupsArgs = {
  hostname: Scalars['String'];
};


export type QueryBackupArgs = {
  hostname: Scalars['String'];
  number: Scalars['Int'];
};


export type QueryHostArgs = {
  hostname: Scalars['String'];
};


export type QueryQueueArgs = {
  state?: Array<Scalars['String']>;
};

export type QueueStats = {
  waiting: Scalars['Int'];
  waitingChildren: Scalars['Int'];
  active: Scalars['Int'];
  failed: Scalars['Int'];
  delayed: Scalars['Int'];
  completed: Scalars['Int'];
  lastExecution?: Maybe<Scalars['Float']>;
  nextWakeup?: Maybe<Scalars['Float']>;
};

export enum QueueTaskState {
  Waiting = 'WAITING',
  Running = 'RUNNING',
  Success = 'SUCCESS',
  Aborted = 'ABORTED',
  Failed = 'FAILED'
}

export type Schedule = {
  activated?: Maybe<Scalars['Boolean']>;
  backupPeriod?: Maybe<Scalars['Float']>;
  backupToKeep?: Maybe<ScheduledBackupToKeep>;
};

export type ScheduledBackupToKeep = {
  hourly?: Maybe<Scalars['Float']>;
  daily?: Maybe<Scalars['Float']>;
  weekly?: Maybe<Scalars['Float']>;
  monthly?: Maybe<Scalars['Float']>;
  yearly?: Maybe<Scalars['Float']>;
};

export type Statistics = {
  diskUsage?: Maybe<DiskUsage>;
  poolUsage?: Maybe<PoolUsage>;
  hosts?: Maybe<Array<HostStatistics>>;
};

export type SubTaskOrGroupTasks = JobSubTask | JobGroupTasks;

export type Subscription = {
  jobUpdated: Job;
  jobWaiting: Scalars['Int'];
  jobFailed: Job;
  jobRemoved: Job;
};

export type TimeSerie = {
  time: Scalars['BigInt'];
  value: Scalars['Float'];
};

export type NavigationBarTasksQueryVariables = Exact<{
  state?: Maybe<Array<Scalars['String']> | Scalars['String']>;
}>;


export type NavigationBarTasksQuery = { queue: Array<(
    Pick<Job, 'id' | 'name' | 'state'>
    & { data: Pick<BackupTask, 'startDate' | 'host' | 'ip' | 'number' | 'groupName' | 'description'> }
  )> };

export type NavigationBarTasksJobUpdatedSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type NavigationBarTasksJobUpdatedSubscription = { jobUpdated: Pick<Job, 'id' | 'name' | 'state'> };

export type RunningTasksMenuQueryVariables = Exact<{
  state?: Maybe<Array<Scalars['String']> | Scalars['String']>;
}>;


export type RunningTasksMenuQuery = { queue: Array<(
    Pick<Job, 'id' | 'name' | 'state'>
    & { data: (
      Pick<BackupTask, 'host'>
      & { progression?: Maybe<Pick<JobProgression, 'fileCount'>> }
    ) }
  )> };

export type RunningTasksMenuJobUpdatedSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type RunningTasksMenuJobUpdatedSubscription = { jobUpdated: (
    Pick<Job, 'id' | 'name' | 'state'>
    & { data: (
      Pick<BackupTask, 'host'>
      & { progression?: Maybe<Pick<JobProgression, 'fileCount'>> }
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


export type DashboardQuery = { queueStats: Pick<QueueStats, 'waiting' | 'active' | 'failed' | 'lastExecution' | 'nextWakeup'> };

export type FragmentFileDescriptionFragment = (
  Pick<FileDescription, 'path' | 'type' | 'symlink'>
  & { stats?: Maybe<Pick<FileStat, 'ownerId' | 'groupId' | 'mode' | 'size' | 'lastModified'>> }
);

type Groups_JobSubTask_Fragment = (
  { __typename: 'JobSubTask' }
  & Pick<JobSubTask, 'taskName' | 'state' | 'description'>
);

type Groups_JobGroupTasks_Fragment = (
  { __typename: 'JobGroupTasks' }
  & Pick<JobGroupTasks, 'groupName' | 'state' | 'description'>
  & { subtasks: Array<{ __typename: 'JobSubTask' } | { __typename: 'JobGroupTasks' }> }
);

export type GroupsFragment = Groups_JobSubTask_Fragment | Groups_JobGroupTasks_Fragment;

export type FragmentJobFragment = (
  Pick<Job, 'id' | 'name' | 'state'>
  & { data: (
    Pick<BackupTask, 'host' | 'number' | 'ip' | 'startDate'>
    & { progression?: Maybe<Pick<JobProgression, 'percent' | 'speed' | 'newFileCount' | 'fileCount'>>, groups: Array<Groups_JobSubTask_Fragment | Groups_JobGroupTasks_Fragment> }
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
