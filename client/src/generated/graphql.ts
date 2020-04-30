export type Maybe<T> = T | null;
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
  files: Array<FileDescription>;
};


export type BackupFilesArgs = {
  path: Scalars['String'];
};

export type BackupQueue = {
  all: Array<Job>;
  waiting: Array<Job>;
  active: Array<Job>;
  failed: Array<Job>;
  delayed: Array<Job>;
  completed: Array<Job>;
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
  number?: Maybe<Scalars['Float']>;
  ip?: Maybe<Scalars['String']>;
  previousDirectory?: Maybe<Scalars['String']>;
  destinationDirectory?: Maybe<Scalars['String']>;
  startDate?: Maybe<Scalars['Float']>;
  subtasks?: Maybe<Array<BackupSubTask>>;
  state?: Maybe<BackupState>;
  progression?: Maybe<TaskProgression>;
};

export type BackupTaskShare = {
  checksum?: Maybe<Scalars['Boolean']>;
  name: Scalars['String'];
  includes?: Maybe<Array<Scalars['String']>>;
  excludes?: Maybe<Array<Scalars['String']>>;
};

export type BtrfsCheck = {
  isBtrfsVolume?: Maybe<Scalars['Boolean']>;
  hasAuthorization?: Maybe<Scalars['Boolean']>;
  backupVolume?: Maybe<Scalars['String']>;
  backupVolumeFileSystem?: Maybe<Scalars['String']>;
  toolsAvailable: BtrfsCheckTools;
};

export type BtrfsCheckTools = {
  btrfstools?: Maybe<Scalars['Boolean']>;
  compsize?: Maybe<Scalars['Boolean']>;
};

export type DhcpAddress = {
  address: Scalars['String'];
  start: Scalars['Float'];
  end: Scalars['Float'];
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

export type ExecuteCommandOperation = {
  name: Scalars['String'];
  command: Scalars['String'];
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

export type Host = {
  name: Scalars['String'];
  configuration: HostConfiguration;
  backups: Array<Backup>;
  lastBackup?: Maybe<Backup>;
};

export type HostConfigOperation = {
  tasks?: Maybe<Array<Operation>>;
  finalizeTasks?: Maybe<Array<Operation>>;
};

export type HostConfiguration = {
  addresses?: Maybe<Array<Scalars['String']>>;
  dhcp?: Maybe<Array<DhcpAddress>>;
  operations?: Maybe<HostConfigOperation>;
  schedule?: Maybe<Schedule>;
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
  progress: Scalars['Float'];
  failedReason?: Maybe<Scalars['String']>;
  stacktrace?: Maybe<Array<Scalars['String']>>;
  state?: Maybe<Scalars['String']>;
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
  number: Scalars['Float'];
  hostname: Scalars['String'];
};

export type Operation = ExecuteCommandOperation | RSyncBackupOperation | RSyncdBackupOperation;

export type Query = {
  backups: Array<Backup>;
  backup: Backup;
  hosts: Array<Host>;
  host: Host;
  queue: BackupQueue;
  status: BtrfsCheck;
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

export type Schedule = {
  activated: Scalars['Boolean'];
  backupPerdiod: Scalars['Float'];
  backupToKeep: ScheduledBackupToKeep;
};

export type ScheduledBackupToKeep = {
  hourly: Scalars['Float'];
  daily: Scalars['Float'];
  weekly: Scalars['Float'];
  monthly: Scalars['Float'];
  yearly: Scalars['Float'];
};

export type Subscription = {
  jobUpdated: Job;
  jobWaiting: Scalars['Int'];
  jobFailed: Job;
  jobRemoved: Job;
};

export type TaskProgression = {
  fileSize: Scalars['Float'];
  newFileSize: Scalars['Float'];
  newFileCount: Scalars['Float'];
  fileCount: Scalars['Float'];
  speed: Scalars['Float'];
  percent: Scalars['Float'];
};


export type FragmentActiveJobFragment = (
  Pick<Job, 'id' | 'state'>
  & { data: (
    Pick<BackupTask, 'host'>
    & { progression?: Maybe<Pick<TaskProgression, 'fileCount' | 'percent'>> }
  ) }
);

export type RunningTasksMenuQueryVariables = {};


export type RunningTasksMenuQuery = { queue: { active: Array<FragmentActiveJobFragment> } };

export type RunningTasksMenuSubSubscriptionVariables = {};


export type RunningTasksMenuSubSubscription = { jobUpdated: FragmentActiveJobFragment };

export type BackupsQueryVariables = {
  hostname: Scalars['String'];
};


export type BackupsQuery = { backups: Array<Pick<Backup, 'number' | 'complete' | 'startDate' | 'endDate' | 'fileCount' | 'newFileCount' | 'existingFileCount' | 'fileSize' | 'newFileSize' | 'existingFileSize' | 'speed'>> };

export type BackupsBrowseQueryVariables = {
  hostname: Scalars['String'];
  number: Scalars['Int'];
  path: Scalars['String'];
};


export type BackupsBrowseQuery = { backup: { files: Array<Pick<FileDescription, 'name' | 'type' | 'mode' | 'size' | 'mtime'>> } };

export type FragmentJobFragment = (
  Pick<Job, 'id' | 'state' | 'failedReason'>
  & { data: (
    Pick<BackupTask, 'host' | 'number' | 'startDate' | 'state'>
    & { progression?: Maybe<Pick<TaskProgression, 'percent' | 'speed' | 'newFileCount' | 'fileCount'>>, subtasks?: Maybe<Array<Pick<BackupSubTask, 'context' | 'description' | 'state'>>> }
  ) }
);

export type HostsQueryVariables = {};


export type HostsQuery = { hosts: Array<(
    Pick<Host, 'name'>
    & { lastBackup?: Maybe<Pick<Backup, 'number' | 'startDate' | 'fileSize' | 'complete'>> }
  )> };

export type RunningTasksQueryVariables = {};


export type RunningTasksQuery = { queue: { all: Array<FragmentJobFragment> } };

export type RunningTasksSubSubscriptionVariables = {};


export type RunningTasksSubSubscription = { jobUpdated: FragmentJobFragment };
