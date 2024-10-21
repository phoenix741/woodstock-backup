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
  DateTime: { input: any; output: any; }
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
  ChecksumChecked = 'ChecksumChecked',
  Delete = 'Delete',
  PoolChecked = 'PoolChecked',
  PoolCleaned = 'PoolCleaned',
  RefcntChecked = 'RefcntChecked',
  Restore = 'Restore'
}

export enum FileManifestAclQualifier {
  GroupId = 'GroupId',
  GroupObj = 'GroupObj',
  Mask = 'Mask',
  Other = 'Other',
  Undefined = 'Undefined',
  UserId = 'UserId',
  UserObj = 'UserObj'
}

export enum HostAvailibilityState {
  Offline = 'Offline',
  Online = 'Online',
  Unknown = 'Unknown'
}

export type QueueListInput = {
  operationName?: InputMaybe<Scalars['String']['input']>;
  queueName?: InputMaybe<Scalars['String']['input']>;
  states?: Array<Scalars['String']['input']>;
};

export enum QueueTaskState {
  Aborted = 'ABORTED',
  Failed = 'FAILED',
  Running = 'RUNNING',
  Success = 'SUCCESS',
  Waiting = 'WAITING'
}

export type HostsQueryVariables = Exact<{ [key: string]: never; }>;


export type HostsQuery = { __typename?: 'Query', hosts: Array<{ __typename?: 'Host', name: string, agentVersion?: string | null, availibilityState?: HostAvailibilityState | null, timeSinceLastBackup?: number | null, dateToNextBackup?: any | null, lastBackupState?: string | null, lastBackup?: { __typename?: 'Backup', number: number, startDate: number, fileSize: bigint, completed: boolean, agentVersion?: string | null } | null, configuration: { __typename?: 'HostConfiguration', schedule?: { __typename?: 'Schedule', activated?: boolean | null } | null } }> };

export type BackupsQueryVariables = Exact<{
  hostname: Scalars['String']['input'];
}>;


export type BackupsQuery = { __typename?: 'Query', backups: Array<{ __typename?: 'Backup', id: string, number: number, completed: boolean, startDate: number, endDate?: number | null, errorCount: number, fileCount: number, newFileCount: number, existingFileCount: number, removedFileCount: number, modifiedFileCount: number, fileSize: bigint, newFileSize: bigint, existingFileSize: bigint, speed: number }> };

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

export type EventsQueryVariables = Exact<{
  firstEvent: Scalars['DateTime']['input'];
  lastEvent: Scalars['DateTime']['input'];
}>;


export type EventsQuery = { __typename?: 'Query', events: Array<{ __typename?: 'ApplicationEvent', uuid: string, type: EventType, step: EventStep, source: EventSource, timestamp: any, errorMessages: Array<string>, status: EventStatus, information?: { __typename: 'EventBackupInformation', hostname: string, number: number, sharePath: Array<string> } | { __typename: 'EventPoolCleanedInformation', size: number, count: number } | { __typename: 'EventPoolInformation', fix: boolean, inUnused: number, inRefcnt: number, inNothing: number, missing: number } | { __typename: 'EventRefCountInformation', fix: boolean, count: number, error: number } | null }> };

export type CleanupPoolMutationVariables = Exact<{ [key: string]: never; }>;


export type CleanupPoolMutation = { __typename?: 'Mutation', cleanupPool: { __typename?: 'JobResponse', id: string } };

export type FsckPoolMutationVariables = Exact<{
  fix: Scalars['Boolean']['input'];
}>;


export type FsckPoolMutation = { __typename?: 'Mutation', checkAndFixPool: { __typename?: 'JobResponse', id: string } };

export type VerifyChecksumMutationVariables = Exact<{ [key: string]: never; }>;


export type VerifyChecksumMutation = { __typename?: 'Mutation', verifyChecksum: { __typename?: 'JobResponse', id: string } };

export type ServerInformationsQueryVariables = Exact<{ [key: string]: never; }>;


export type ServerInformationsQuery = { __typename?: 'Query', informations: { __typename?: 'ServerInformations', platform: string, uptime: number, hostname: string, woodstockVersion?: string | null } };

export type DiskUsageStatisticsQueryVariables = Exact<{ [key: string]: never; }>;


export type DiskUsageStatisticsQuery = { __typename?: 'Query', statistics: { __typename?: 'Statistics', hosts?: Array<{ __typename?: 'HostStatistics', host?: string | null, size?: bigint | null, compressedSize?: bigint | null }> | null } };

export type PoolStatisticsQueryVariables = Exact<{ [key: string]: never; }>;


export type PoolStatisticsQuery = { __typename?: 'Query', statistics: { __typename?: 'Statistics', diskUsage?: { __typename?: 'DiskUsage', used?: bigint | null, usedLastMonth?: bigint | null, free?: bigint | null, total?: bigint | null } | null, poolUsage?: { __typename?: 'PoolUsage', nbChunk?: number | null, nbChunkLastMonth?: number | null, nbRef?: number | null, nbRefLastMonth?: number | null, size?: bigint | null, compressedSize?: bigint | null, compressedSizeLastMonth?: bigint | null, unusedSize?: bigint | null, nbChunkRange?: Array<{ __typename?: 'NumberTimeSerie', time: number, value: number }> | null, compressedSizeRange?: Array<{ __typename?: 'BigIntTimeSerie', time: number, value: bigint }> | null } | null } };

export type QueueStatisticsQueryVariables = Exact<{ [key: string]: never; }>;


export type QueueStatisticsQuery = { __typename?: 'Query', queueStats: { __typename?: 'QueueStats', active: number, waiting: number, failed: number, delayed: number, completed: number } };

export type ProgressTaskFragment = { __typename?: 'JobProgression', progressCurrent?: bigint | null, progressMax?: bigint | null, fileSize?: bigint | null, newFileSize?: bigint | null, compressedFileSize?: bigint | null, newCompressedFileSize?: bigint | null, fileCount?: number | null, newFileCount?: number | null, errorCount?: number | null, percent?: number | null } & { ' $fragmentName'?: 'ProgressTaskFragment' };

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

export type JobProgressionFragment = { __typename?: 'JobProgression', newFileCount?: number | null, fileCount?: number | null, progressCurrent?: bigint | null, progressMax?: bigint | null, percent?: number | null, speed?: number | null } & { ' $fragmentName'?: 'JobProgressionFragment' };

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
export const JobProgressionFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobProgression"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobProgression"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}}]}}]} as unknown as DocumentNode<JobProgressionFragment, unknown>;
export const ProgressTaskFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ProgressTask"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobProgression"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}}]}}]} as unknown as DocumentNode<ProgressTaskFragment, unknown>;
export const TaskDescriptionFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"TaskDescription"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"SubTaskOrGroupTasks"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobSubTask"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"taskName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"state"}}]}}]}}]} as unknown as DocumentNode<TaskDescriptionFragment, unknown>;
export const BackupTaskFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BackupTask"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"SubTaskOrGroupTasks"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobGroupTasks"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"groupName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProgressTask"}}]}},{"kind":"Field","alias":{"kind":"Name","value":"taskDescription"},"name":{"kind":"Name","value":"subtasks"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"TaskDescription"}}]}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobSubTask"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"taskName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProgressTask"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ProgressTask"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobProgression"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"TaskDescription"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"SubTaskOrGroupTasks"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobSubTask"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"taskName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"state"}}]}}]}}]} as unknown as DocumentNode<BackupTaskFragment, unknown>;
export const JobFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Job"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Job"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"queueName"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"failedReason"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"groupName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"ip"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobProgression"}}]}},{"kind":"Field","name":{"kind":"Name","value":"subtasks"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BackupTask"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ProgressTask"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobProgression"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"TaskDescription"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"SubTaskOrGroupTasks"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobSubTask"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"taskName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"state"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobProgression"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobProgression"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BackupTask"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"SubTaskOrGroupTasks"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobGroupTasks"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"groupName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProgressTask"}}]}},{"kind":"Field","alias":{"kind":"Name","value":"taskDescription"},"name":{"kind":"Name","value":"subtasks"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"TaskDescription"}}]}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobSubTask"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"taskName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProgressTask"}}]}}]}}]}}]} as unknown as DocumentNode<JobFragment, unknown>;
export const HostsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"Hosts"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"hosts"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"lastBackup"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"agentVersion"}}]}},{"kind":"Field","name":{"kind":"Name","value":"agentVersion"}},{"kind":"Field","name":{"kind":"Name","value":"availibilityState"}},{"kind":"Field","name":{"kind":"Name","value":"timeSinceLastBackup"}},{"kind":"Field","name":{"kind":"Name","value":"dateToNextBackup"}},{"kind":"Field","name":{"kind":"Name","value":"lastBackupState"}},{"kind":"Field","name":{"kind":"Name","value":"configuration"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"schedule"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"activated"}}]}}]}}]}}]}}]} as unknown as DocumentNode<HostsQuery, HostsQueryVariables>;
export const BackupsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"Backups"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"backups"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"hostname"},"value":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"endDate"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"existingFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"removedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"modifiedFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"existingFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}}]}}]}}]} as unknown as DocumentNode<BackupsQuery, BackupsQueryVariables>;
export const BackupsBrowseDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"BackupsBrowse"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"number"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"sharePath"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"path"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"backup"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"hostname"},"value":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}}},{"kind":"Argument","name":{"kind":"Name","value":"number"},"value":{"kind":"Variable","name":{"kind":"Name","value":"number"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"files"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"sharePath"},"value":{"kind":"Variable","name":{"kind":"Name","value":"sharePath"}}},{"kind":"Argument","name":{"kind":"Name","value":"path"},"value":{"kind":"Variable","name":{"kind":"Name","value":"path"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"FragmentFileDescription"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"FragmentFileDescription"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"FileDescription"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"path"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"stats"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"ownerId"}},{"kind":"Field","name":{"kind":"Name","value":"groupId"}},{"kind":"Field","name":{"kind":"Name","value":"mode"}},{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"lastModified"}}]}},{"kind":"Field","name":{"kind":"Name","value":"symlink"}}]}}]} as unknown as DocumentNode<BackupsBrowseQuery, BackupsBrowseQueryVariables>;
export const CreateBackupDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"createBackup"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"createBackup"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"hostname"},"value":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]} as unknown as DocumentNode<CreateBackupMutation, CreateBackupMutationVariables>;
export const RemoveBackupDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"removeBackup"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"number"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"removeBackup"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"hostname"},"value":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}}},{"kind":"Argument","name":{"kind":"Name","value":"number"},"value":{"kind":"Variable","name":{"kind":"Name","value":"number"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]} as unknown as DocumentNode<RemoveBackupMutation, RemoveBackupMutationVariables>;
export const SharesBrowseDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"SharesBrowse"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"number"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"backup"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"hostname"},"value":{"kind":"Variable","name":{"kind":"Name","value":"hostname"}}},{"kind":"Argument","name":{"kind":"Name","value":"number"},"value":{"kind":"Variable","name":{"kind":"Name","value":"number"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"shares"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"FragmentFileDescription"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"FragmentFileDescription"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"FileDescription"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"path"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"stats"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"ownerId"}},{"kind":"Field","name":{"kind":"Name","value":"groupId"}},{"kind":"Field","name":{"kind":"Name","value":"mode"}},{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"lastModified"}}]}},{"kind":"Field","name":{"kind":"Name","value":"symlink"}}]}}]} as unknown as DocumentNode<SharesBrowseQuery, SharesBrowseQueryVariables>;
export const EventsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"Events"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"firstEvent"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"DateTime"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"lastEvent"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"DateTime"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"events"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"firstEvent"},"value":{"kind":"Variable","name":{"kind":"Name","value":"firstEvent"}}},{"kind":"Argument","name":{"kind":"Name","value":"lastEvent"},"value":{"kind":"Variable","name":{"kind":"Name","value":"lastEvent"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"uuid"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"step"}},{"kind":"Field","name":{"kind":"Name","value":"source"}},{"kind":"Field","name":{"kind":"Name","value":"timestamp"}},{"kind":"Field","name":{"kind":"Name","value":"errorMessages"}},{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"information"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventBackupInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"hostname"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"sharePath"}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventRefCountInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"fix"}},{"kind":"Field","name":{"kind":"Name","value":"count"}},{"kind":"Field","name":{"kind":"Name","value":"error"}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventPoolInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"fix"}},{"kind":"Field","name":{"kind":"Name","value":"inUnused"}},{"kind":"Field","name":{"kind":"Name","value":"inRefcnt"}},{"kind":"Field","name":{"kind":"Name","value":"inNothing"}},{"kind":"Field","name":{"kind":"Name","value":"missing"}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"EventPoolCleanedInformation"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"count"}}]}}]}}]}}]}}]} as unknown as DocumentNode<EventsQuery, EventsQueryVariables>;
export const CleanupPoolDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"cleanupPool"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"cleanupPool"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]} as unknown as DocumentNode<CleanupPoolMutation, CleanupPoolMutationVariables>;
export const FsckPoolDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"fsckPool"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"fix"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Boolean"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"checkAndFixPool"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"fix"},"value":{"kind":"Variable","name":{"kind":"Name","value":"fix"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]} as unknown as DocumentNode<FsckPoolMutation, FsckPoolMutationVariables>;
export const VerifyChecksumDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"verifyChecksum"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"verifyChecksum"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]} as unknown as DocumentNode<VerifyChecksumMutation, VerifyChecksumMutationVariables>;
export const ServerInformationsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"ServerInformations"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"informations"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"platform"}},{"kind":"Field","name":{"kind":"Name","value":"uptime"}},{"kind":"Field","name":{"kind":"Name","value":"hostname"}},{"kind":"Field","name":{"kind":"Name","value":"woodstockVersion"}}]}}]}}]} as unknown as DocumentNode<ServerInformationsQuery, ServerInformationsQueryVariables>;
export const DiskUsageStatisticsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"DiskUsageStatistics"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"statistics"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"hosts"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"compressedSize"}}]}}]}}]}}]} as unknown as DocumentNode<DiskUsageStatisticsQuery, DiskUsageStatisticsQueryVariables>;
export const PoolStatisticsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"PoolStatistics"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"statistics"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"diskUsage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"used"}},{"kind":"Field","name":{"kind":"Name","value":"usedLastMonth"}},{"kind":"Field","name":{"kind":"Name","value":"free"}},{"kind":"Field","name":{"kind":"Name","value":"total"}}]}},{"kind":"Field","name":{"kind":"Name","value":"poolUsage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"nbChunk"}},{"kind":"Field","name":{"kind":"Name","value":"nbChunkLastMonth"}},{"kind":"Field","name":{"kind":"Name","value":"nbChunkRange"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"time"}},{"kind":"Field","name":{"kind":"Name","value":"value"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nbRef"}},{"kind":"Field","name":{"kind":"Name","value":"nbRefLastMonth"}},{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"compressedSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedSizeLastMonth"}},{"kind":"Field","name":{"kind":"Name","value":"compressedSizeRange"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"time"}},{"kind":"Field","name":{"kind":"Name","value":"value"}}]}},{"kind":"Field","name":{"kind":"Name","value":"unusedSize"}}]}}]}}]}}]} as unknown as DocumentNode<PoolStatisticsQuery, PoolStatisticsQueryVariables>;
export const QueueStatisticsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"QueueStatistics"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"queueStats"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"active"}},{"kind":"Field","name":{"kind":"Name","value":"waiting"}},{"kind":"Field","name":{"kind":"Name","value":"failed"}},{"kind":"Field","name":{"kind":"Name","value":"delayed"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}}]}}]}}]} as unknown as DocumentNode<QueueStatisticsQuery, QueueStatisticsQueryVariables>;
export const TasksDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"Tasks"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"QueueListInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"queue"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Job"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobProgression"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobProgression"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ProgressTask"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobProgression"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"TaskDescription"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"SubTaskOrGroupTasks"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobSubTask"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"taskName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"state"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BackupTask"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"SubTaskOrGroupTasks"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobGroupTasks"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"groupName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProgressTask"}}]}},{"kind":"Field","alias":{"kind":"Name","value":"taskDescription"},"name":{"kind":"Name","value":"subtasks"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"TaskDescription"}}]}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobSubTask"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"taskName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProgressTask"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Job"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Job"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"queueName"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"failedReason"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"groupName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"ip"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobProgression"}}]}},{"kind":"Field","name":{"kind":"Name","value":"subtasks"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BackupTask"}}]}}]}}]}}]} as unknown as DocumentNode<TasksQuery, TasksQueryVariables>;
export const QueueTasksJobUpdatedDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"subscription","name":{"kind":"Name","value":"QueueTasksJobUpdated"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"jobUpdated"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Job"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"JobProgression"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobProgression"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}},{"kind":"Field","name":{"kind":"Name","value":"speed"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ProgressTask"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobProgression"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressCurrent"}},{"kind":"Field","name":{"kind":"Name","value":"progressMax"}},{"kind":"Field","name":{"kind":"Name","value":"fileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"compressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"newCompressedFileSize"}},{"kind":"Field","name":{"kind":"Name","value":"fileCount"}},{"kind":"Field","name":{"kind":"Name","value":"newFileCount"}},{"kind":"Field","name":{"kind":"Name","value":"errorCount"}},{"kind":"Field","name":{"kind":"Name","value":"percent"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"TaskDescription"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"SubTaskOrGroupTasks"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobSubTask"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"taskName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"state"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BackupTask"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"SubTaskOrGroupTasks"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobGroupTasks"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"groupName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProgressTask"}}]}},{"kind":"Field","alias":{"kind":"Name","value":"taskDescription"},"name":{"kind":"Name","value":"subtasks"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"TaskDescription"}}]}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"JobSubTask"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"taskName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProgressTask"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Job"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Job"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"queueName"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"failedReason"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"host"}},{"kind":"Field","name":{"kind":"Name","value":"number"}},{"kind":"Field","name":{"kind":"Name","value":"startDate"}},{"kind":"Field","name":{"kind":"Name","value":"groupName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"ip"}},{"kind":"Field","name":{"kind":"Name","value":"state"}},{"kind":"Field","name":{"kind":"Name","value":"progression"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"JobProgression"}}]}},{"kind":"Field","name":{"kind":"Name","value":"subtasks"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BackupTask"}}]}}]}}]}}]} as unknown as DocumentNode<QueueTasksJobUpdatedSubscription, QueueTasksJobUpdatedSubscriptionVariables>;
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
