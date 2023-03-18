/* eslint-disable */
import * as types from './graphql';
import type { TypedDocumentNode as DocumentNode } from '@graphql-typed-document-node/core';

/**
 * Map of all GraphQL operations in the project.
 *
 * This map has several performance disadvantages:
 * 1. It is not tree-shakeable, so it will include all operations in the project.
 * 2. It is not minifiable, so the string of a GraphQL query will be multiple times inside the bundle.
 * 3. It does not support dead code elimination, so it will add unused operations.
 *
 * Therefore it is highly recommended to use the babel or swc plugin for production.
 */
const documents = {
    "query Hosts {\n  hosts {\n    name\n    lastBackup {\n      number\n      startDate\n      fileSize\n      complete\n    }\n    lastBackupState\n    configuration {\n      schedule {\n        activated\n      }\n    }\n  }\n}": types.HostsDocument,
    "query Backups($hostname: String!) {\n  backups(hostname: $hostname) {\n    number\n    complete\n    startDate\n    endDate\n    fileCount\n    newFileCount\n    existingFileCount\n    fileSize\n    newFileSize\n    existingFileSize\n    speed\n  }\n}": types.BackupsDocument,
    "query BackupsBrowse($hostname: String!, $number: Int!, $sharePath: String!, $path: String!) {\n  backup(hostname: $hostname, number: $number) {\n    files(sharePath: $sharePath, path: $path) {\n      ...FragmentFileDescription\n    }\n  }\n}": types.BackupsBrowseDocument,
    "mutation createBackup($hostname: String!) {\n  createBackup(hostname: $hostname) {\n    id\n  }\n}": types.CreateBackupDocument,
    "mutation removeBackup($hostname: String!, $number: Int!) {\n  removeBackup(hostname: $hostname, number: $number) {\n    id\n  }\n}": types.RemoveBackupDocument,
    "fragment FragmentFileDescription on FileDescription {\n  path\n  type\n  stats {\n    ownerId\n    groupId\n    mode\n    size\n    lastModified\n  }\n  symlink\n}": types.FragmentFileDescriptionFragmentDoc,
    "query SharesBrowse($hostname: String!, $number: Int!) {\n  backup(hostname: $hostname, number: $number) {\n    shares {\n      ...FragmentFileDescription\n    }\n  }\n}": types.SharesBrowseDocument,
    "mutation cleanupPool {\n  cleanupPool {\n    id\n  }\n}": types.CleanupPoolDocument,
    "mutation fsckPool($fix: Boolean!) {\n  checkAndFixPool(fix: $fix) {\n    id\n  }\n}": types.FsckPoolDocument,
    "mutation verifyChecksum {\n  verifyChecksum {\n    id\n  }\n}": types.VerifyChecksumDocument,
    "query DiskUsageStatistics {\n  statistics {\n    hosts {\n      host\n      size\n      compressedSize\n    }\n  }\n}": types.DiskUsageStatisticsDocument,
    "query PoolStatistics {\n  statistics {\n    diskUsage {\n      used\n      usedLastMonth\n      free\n      total\n    }\n    poolUsage {\n      nbChunk\n      nbChunkLastMonth\n      nbChunkRange {\n        time\n        value\n      }\n      nbRef\n      nbRefLastMonth\n      size\n      compressedSize\n      compressedSizeLastMonth\n      compressedSizeRange {\n        time\n        value\n      }\n      unusedSize\n    }\n  }\n}": types.PoolStatisticsDocument,
    "query QueueStatistics {\n  queueStats {\n    active\n    waiting\n    failed\n    delayed\n    completed\n  }\n}": types.QueueStatisticsDocument,
    "fragment ProgressTask on JobProgression {\n  progressCurrent\n  progressMax\n  fileSize\n  newFileSize\n  compressedFileSize\n  newCompressedFileSize\n  fileCount\n  newFileCount\n  errorCount\n}\n\nfragment TaskDescription on SubTaskOrGroupTasks {\n  __typename\n  ... on JobSubTask {\n    taskName\n    description\n    state\n  }\n}\n\nfragment BackupTask on SubTaskOrGroupTasks {\n  __typename\n  ... on JobGroupTasks {\n    groupName\n    description\n    state\n    progression {\n      ...ProgressTask\n    }\n    taskDescription: subtasks {\n      ...TaskDescription\n    }\n  }\n  ... on JobSubTask {\n    taskName\n    description\n    state\n    progression {\n      ...ProgressTask\n    }\n  }\n}\n\nfragment JobProgression on JobProgression {\n  newFileCount\n  fileCount\n  progressCurrent\n  progressMax\n  speed\n}\n\nfragment Job on Job {\n  id\n  queueName\n  name\n  failedReason\n  state\n  data {\n    host\n    number\n    startDate\n    groupName\n    description\n    ip\n    state\n    progression {\n      ...JobProgression\n    }\n    subtasks {\n      ...BackupTask\n    }\n  }\n}\n\nquery Tasks($input: QueueListInput!) {\n  queue(input: $input) {\n    ...Job\n  }\n}\n\nsubscription QueueTasksJobUpdated {\n  jobUpdated {\n    ...Job\n  }\n}": types.ProgressTaskFragmentDoc,
};

/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 *
 *
 * @example
 * ```ts
 * const query = graphql(`query GetUser($id: ID!) { user(id: $id) { name } }`);
 * ```
 *
 * The query argument is unknown!
 * Please regenerate the types.
 */
export function graphql(source: string): unknown;

/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "query Hosts {\n  hosts {\n    name\n    lastBackup {\n      number\n      startDate\n      fileSize\n      complete\n    }\n    lastBackupState\n    configuration {\n      schedule {\n        activated\n      }\n    }\n  }\n}"): (typeof documents)["query Hosts {\n  hosts {\n    name\n    lastBackup {\n      number\n      startDate\n      fileSize\n      complete\n    }\n    lastBackupState\n    configuration {\n      schedule {\n        activated\n      }\n    }\n  }\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "query Backups($hostname: String!) {\n  backups(hostname: $hostname) {\n    number\n    complete\n    startDate\n    endDate\n    fileCount\n    newFileCount\n    existingFileCount\n    fileSize\n    newFileSize\n    existingFileSize\n    speed\n  }\n}"): (typeof documents)["query Backups($hostname: String!) {\n  backups(hostname: $hostname) {\n    number\n    complete\n    startDate\n    endDate\n    fileCount\n    newFileCount\n    existingFileCount\n    fileSize\n    newFileSize\n    existingFileSize\n    speed\n  }\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "query BackupsBrowse($hostname: String!, $number: Int!, $sharePath: String!, $path: String!) {\n  backup(hostname: $hostname, number: $number) {\n    files(sharePath: $sharePath, path: $path) {\n      ...FragmentFileDescription\n    }\n  }\n}"): (typeof documents)["query BackupsBrowse($hostname: String!, $number: Int!, $sharePath: String!, $path: String!) {\n  backup(hostname: $hostname, number: $number) {\n    files(sharePath: $sharePath, path: $path) {\n      ...FragmentFileDescription\n    }\n  }\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "mutation createBackup($hostname: String!) {\n  createBackup(hostname: $hostname) {\n    id\n  }\n}"): (typeof documents)["mutation createBackup($hostname: String!) {\n  createBackup(hostname: $hostname) {\n    id\n  }\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "mutation removeBackup($hostname: String!, $number: Int!) {\n  removeBackup(hostname: $hostname, number: $number) {\n    id\n  }\n}"): (typeof documents)["mutation removeBackup($hostname: String!, $number: Int!) {\n  removeBackup(hostname: $hostname, number: $number) {\n    id\n  }\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "fragment FragmentFileDescription on FileDescription {\n  path\n  type\n  stats {\n    ownerId\n    groupId\n    mode\n    size\n    lastModified\n  }\n  symlink\n}"): (typeof documents)["fragment FragmentFileDescription on FileDescription {\n  path\n  type\n  stats {\n    ownerId\n    groupId\n    mode\n    size\n    lastModified\n  }\n  symlink\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "query SharesBrowse($hostname: String!, $number: Int!) {\n  backup(hostname: $hostname, number: $number) {\n    shares {\n      ...FragmentFileDescription\n    }\n  }\n}"): (typeof documents)["query SharesBrowse($hostname: String!, $number: Int!) {\n  backup(hostname: $hostname, number: $number) {\n    shares {\n      ...FragmentFileDescription\n    }\n  }\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "mutation cleanupPool {\n  cleanupPool {\n    id\n  }\n}"): (typeof documents)["mutation cleanupPool {\n  cleanupPool {\n    id\n  }\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "mutation fsckPool($fix: Boolean!) {\n  checkAndFixPool(fix: $fix) {\n    id\n  }\n}"): (typeof documents)["mutation fsckPool($fix: Boolean!) {\n  checkAndFixPool(fix: $fix) {\n    id\n  }\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "mutation verifyChecksum {\n  verifyChecksum {\n    id\n  }\n}"): (typeof documents)["mutation verifyChecksum {\n  verifyChecksum {\n    id\n  }\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "query DiskUsageStatistics {\n  statistics {\n    hosts {\n      host\n      size\n      compressedSize\n    }\n  }\n}"): (typeof documents)["query DiskUsageStatistics {\n  statistics {\n    hosts {\n      host\n      size\n      compressedSize\n    }\n  }\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "query PoolStatistics {\n  statistics {\n    diskUsage {\n      used\n      usedLastMonth\n      free\n      total\n    }\n    poolUsage {\n      nbChunk\n      nbChunkLastMonth\n      nbChunkRange {\n        time\n        value\n      }\n      nbRef\n      nbRefLastMonth\n      size\n      compressedSize\n      compressedSizeLastMonth\n      compressedSizeRange {\n        time\n        value\n      }\n      unusedSize\n    }\n  }\n}"): (typeof documents)["query PoolStatistics {\n  statistics {\n    diskUsage {\n      used\n      usedLastMonth\n      free\n      total\n    }\n    poolUsage {\n      nbChunk\n      nbChunkLastMonth\n      nbChunkRange {\n        time\n        value\n      }\n      nbRef\n      nbRefLastMonth\n      size\n      compressedSize\n      compressedSizeLastMonth\n      compressedSizeRange {\n        time\n        value\n      }\n      unusedSize\n    }\n  }\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "query QueueStatistics {\n  queueStats {\n    active\n    waiting\n    failed\n    delayed\n    completed\n  }\n}"): (typeof documents)["query QueueStatistics {\n  queueStats {\n    active\n    waiting\n    failed\n    delayed\n    completed\n  }\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "fragment ProgressTask on JobProgression {\n  progressCurrent\n  progressMax\n  fileSize\n  newFileSize\n  compressedFileSize\n  newCompressedFileSize\n  fileCount\n  newFileCount\n  errorCount\n}\n\nfragment TaskDescription on SubTaskOrGroupTasks {\n  __typename\n  ... on JobSubTask {\n    taskName\n    description\n    state\n  }\n}\n\nfragment BackupTask on SubTaskOrGroupTasks {\n  __typename\n  ... on JobGroupTasks {\n    groupName\n    description\n    state\n    progression {\n      ...ProgressTask\n    }\n    taskDescription: subtasks {\n      ...TaskDescription\n    }\n  }\n  ... on JobSubTask {\n    taskName\n    description\n    state\n    progression {\n      ...ProgressTask\n    }\n  }\n}\n\nfragment JobProgression on JobProgression {\n  newFileCount\n  fileCount\n  progressCurrent\n  progressMax\n  speed\n}\n\nfragment Job on Job {\n  id\n  queueName\n  name\n  failedReason\n  state\n  data {\n    host\n    number\n    startDate\n    groupName\n    description\n    ip\n    state\n    progression {\n      ...JobProgression\n    }\n    subtasks {\n      ...BackupTask\n    }\n  }\n}\n\nquery Tasks($input: QueueListInput!) {\n  queue(input: $input) {\n    ...Job\n  }\n}\n\nsubscription QueueTasksJobUpdated {\n  jobUpdated {\n    ...Job\n  }\n}"): (typeof documents)["fragment ProgressTask on JobProgression {\n  progressCurrent\n  progressMax\n  fileSize\n  newFileSize\n  compressedFileSize\n  newCompressedFileSize\n  fileCount\n  newFileCount\n  errorCount\n}\n\nfragment TaskDescription on SubTaskOrGroupTasks {\n  __typename\n  ... on JobSubTask {\n    taskName\n    description\n    state\n  }\n}\n\nfragment BackupTask on SubTaskOrGroupTasks {\n  __typename\n  ... on JobGroupTasks {\n    groupName\n    description\n    state\n    progression {\n      ...ProgressTask\n    }\n    taskDescription: subtasks {\n      ...TaskDescription\n    }\n  }\n  ... on JobSubTask {\n    taskName\n    description\n    state\n    progression {\n      ...ProgressTask\n    }\n  }\n}\n\nfragment JobProgression on JobProgression {\n  newFileCount\n  fileCount\n  progressCurrent\n  progressMax\n  speed\n}\n\nfragment Job on Job {\n  id\n  queueName\n  name\n  failedReason\n  state\n  data {\n    host\n    number\n    startDate\n    groupName\n    description\n    ip\n    state\n    progression {\n      ...JobProgression\n    }\n    subtasks {\n      ...BackupTask\n    }\n  }\n}\n\nquery Tasks($input: QueueListInput!) {\n  queue(input: $input) {\n    ...Job\n  }\n}\n\nsubscription QueueTasksJobUpdated {\n  jobUpdated {\n    ...Job\n  }\n}"];

export function graphql(source: string) {
  return (documents as any)[source] ?? {};
}

export type DocumentType<TDocumentNode extends DocumentNode<any, any>> = TDocumentNode extends DocumentNode<  infer TType,  any>  ? TType  : never;