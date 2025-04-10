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
 * Learn more about it here: https://the-guild.dev/graphql/codegen/plugins/presets/preset-client#reducing-bundle-size
 */
const documents = {
    "\n  fragment ApplicationEvent on ApplicationEvent {\n    uuid\n    type\n    step\n    source\n    timestamp\n    errorMessages\n    status\n    information {\n      __typename\n      ... on EventBackupInformation {\n        ...EventBackupInformation\n      }\n      ... on EventRefCountInformation {\n        ...EventRefCountInformation\n      }\n      ... on EventPoolInformation {\n        ...EventPoolInformation\n      }\n      ... on EventPoolCleanedInformation {\n        ...EventPoolCleanedInformation\n      }\n      ... on EventHashConversionInformation {\n        ...EventHashConversionInformation\n      }\n    }\n  }\n": types.ApplicationEventFragmentDoc,
    "\n  fragment EventBackupInformation on EventBackupInformation {\n    hostname\n    number\n    sharePath\n  }\n": types.EventBackupInformationFragmentDoc,
    "\n  fragment EventRefCountInformation on EventRefCountInformation {\n    fix\n    count\n    error\n  }\n": types.EventRefCountInformationFragmentDoc,
    "\n  fragment EventPoolInformation on EventPoolInformation {\n    fix\n    inUnused\n    inRefcnt\n    inNothing\n    missing\n  }\n": types.EventPoolInformationFragmentDoc,
    "\n  fragment EventPoolCleanedInformation on EventPoolCleanedInformation {\n    size\n    count\n  }\n": types.EventPoolCleanedInformationFragmentDoc,
    "\n  fragment EventHashConversionInformation on EventHashConversionInformation {\n    count\n    algorithm\n  }\n": types.EventHashConversionInformationFragmentDoc,
    "query Hosts {\n  hosts {\n    name\n    lastBackup {\n      number\n      startDate\n      fileSize\n      completed\n      agentVersion\n    }\n    agentVersion\n    availibilityState\n    timeSinceLastBackup\n    dateToNextBackup\n    lastBackupState\n    configuration {\n      schedule {\n        activated\n      }\n    }\n  }\n}": types.HostsDocument,
    "query Backups($hostname: String!) {\n  backups(hostname: $hostname) {\n    id\n    number\n    completed\n    startDate\n    endDate\n    errorCount\n    fileCount\n    newFileCount\n    existingFileCount\n    removedFileCount\n    modifiedFileCount\n    fileSize\n    newFileSize\n    existingFileSize\n    speed\n  }\n}": types.BackupsDocument,
    "query BackupsBrowse($hostname: String!, $number: Int!, $sharePath: String!, $path: String!) {\n  backup(hostname: $hostname, number: $number) {\n    id\n    files(sharePath: $sharePath, path: $path) {\n      ...FragmentFileDescription\n    }\n  }\n}": types.BackupsBrowseDocument,
    "mutation createBackup($hostname: String!) {\n  createBackup(hostname: $hostname) {\n    id\n  }\n}": types.CreateBackupDocument,
    "mutation removeBackup($hostname: String!, $number: Int!) {\n  removeBackup(hostname: $hostname, number: $number) {\n    id\n  }\n}": types.RemoveBackupDocument,
    "fragment FragmentFileDescription on FileDescription {\n  path\n  type\n  stats {\n    ownerId\n    groupId\n    mode\n    size\n    lastModified\n  }\n  symlink\n}": types.FragmentFileDescriptionFragmentDoc,
    "query SharesBrowse($hostname: String!, $number: Int!) {\n  backup(hostname: $hostname, number: $number) {\n    id\n    shares {\n      ...FragmentFileDescription\n    }\n  }\n}": types.SharesBrowseDocument,
    "\n  fragment JobPoolResponse on JobResponse {\n    id\n  }\n": types.JobPoolResponseFragmentDoc,
    "\n      mutation restoreBackup($input: RestoreInput!) {\n        restoreBackup(input: $input) {\n          ...JobPoolResponse\n        }\n      }\n    ": types.RestoreBackupDocument,
    "\n      mutation clearCache {\n        clearCache {\n          void\n        }\n      }\n    ": types.ClearCacheDocument,
    "\n  query Events($firstEvent: DateTime!, $lastEvent: DateTime!) {\n    events(firstEvent: $firstEvent, lastEvent: $lastEvent) {\n      ...ApplicationEvent\n    }\n  }\n": types.EventsDocument,
    "\n      mutation cleanupPool {\n        cleanupPool {\n          ...JobPoolResponse\n        }\n      }\n    ": types.CleanupPoolDocument,
    "\n      mutation fsckPool($fix: Boolean!) {\n        checkAndFixPool(fix: $fix) {\n          ...JobPoolResponse\n        }\n      }\n    ": types.FsckPoolDocument,
    "\n      mutation verifyChecksum {\n        verifyChecksum {\n          ...JobPoolResponse\n        }\n      }\n    ": types.VerifyChecksumDocument,
    "\n      query ServerInformations {\n        informations {\n          platform\n          uptime\n          hostname\n          woodstockVersion\n        }\n      }\n    ": types.ServerInformationsDocument,
    "\n      query DiskUsageStatistics {\n        statistics {\n          hosts {\n            host\n\n            size\n            compressedSize\n          }\n        }\n      }\n    ": types.DiskUsageStatisticsDocument,
    "\n      query QueueStatistics {\n        queueStats {\n          active\n          waiting\n          failed\n          delayed\n          completed\n        }\n      }\n    ": types.QueueStatisticsDocument,
    "\n      query PoolStatistics {\n        statistics {\n          diskUsage {\n            used\n            usedLastMonth\n            free\n            total\n          }\n          poolUsage {\n            nbChunk\n            nbChunkLastMonth\n            nbChunkRange {\n              time\n              value\n            }\n\n            nbRef\n            nbRefLastMonth\n\n            size\n            compressedSize\n            compressedSizeLastMonth\n            compressedSizeRange {\n              time\n              value\n            }\n\n            unusedSize\n          }\n        }\n      }\n    ": types.PoolStatisticsDocument,
    "\n  fragment ProgressTask on JobProgression {\n    progressCurrent\n    progressMax\n\n    fileSize\n    newFileSize\n\n    compressedFileSize\n    newCompressedFileSize\n\n    fileCount\n    newFileCount\n\n    errorCount\n\n    percent\n    speed\n  }\n": types.ProgressTaskFragmentDoc,
    "\n  fragment TaskDescription on SubTaskOrGroupTasks {\n    __typename\n    ... on JobSubTask {\n      taskName\n      description\n      state\n    }\n  }\n": types.TaskDescriptionFragmentDoc,
    "\n  fragment BackupTask on SubTaskOrGroupTasks {\n    __typename\n    ... on JobGroupTasks {\n      groupName\n      description\n      state\n      progression {\n        ...ProgressTask\n      }\n      taskDescription: subtasks {\n        ...TaskDescription\n      }\n    }\n    ... on JobSubTask {\n      taskName\n      description\n      state\n      progression {\n        ...ProgressTask\n      }\n    }\n  }\n": types.BackupTaskFragmentDoc,
    "\n  fragment Job on Job {\n    id\n    queueName\n    name\n    failedReason\n    state\n    data {\n      host\n      number\n      startDate\n      groupName\n      description\n      ip\n      state\n      progression {\n        ...ProgressTask\n      }\n      subtasks {\n        ...BackupTask\n      }\n    }\n  }\n": types.JobFragmentDoc,
    "\n      query Tasks($input: QueueListInput!) {\n        queue(input: $input) {\n          ...Job\n        }\n      }\n    ": types.TasksDocument,
    "\n      subscription QueueTasksJobUpdated {\n        jobUpdated {\n          ...Job\n        }\n      }\n    ": types.QueueTasksJobUpdatedDocument,
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
export function graphql(source: "\n  fragment ApplicationEvent on ApplicationEvent {\n    uuid\n    type\n    step\n    source\n    timestamp\n    errorMessages\n    status\n    information {\n      __typename\n      ... on EventBackupInformation {\n        ...EventBackupInformation\n      }\n      ... on EventRefCountInformation {\n        ...EventRefCountInformation\n      }\n      ... on EventPoolInformation {\n        ...EventPoolInformation\n      }\n      ... on EventPoolCleanedInformation {\n        ...EventPoolCleanedInformation\n      }\n      ... on EventHashConversionInformation {\n        ...EventHashConversionInformation\n      }\n    }\n  }\n"): (typeof documents)["\n  fragment ApplicationEvent on ApplicationEvent {\n    uuid\n    type\n    step\n    source\n    timestamp\n    errorMessages\n    status\n    information {\n      __typename\n      ... on EventBackupInformation {\n        ...EventBackupInformation\n      }\n      ... on EventRefCountInformation {\n        ...EventRefCountInformation\n      }\n      ... on EventPoolInformation {\n        ...EventPoolInformation\n      }\n      ... on EventPoolCleanedInformation {\n        ...EventPoolCleanedInformation\n      }\n      ... on EventHashConversionInformation {\n        ...EventHashConversionInformation\n      }\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment EventBackupInformation on EventBackupInformation {\n    hostname\n    number\n    sharePath\n  }\n"): (typeof documents)["\n  fragment EventBackupInformation on EventBackupInformation {\n    hostname\n    number\n    sharePath\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment EventRefCountInformation on EventRefCountInformation {\n    fix\n    count\n    error\n  }\n"): (typeof documents)["\n  fragment EventRefCountInformation on EventRefCountInformation {\n    fix\n    count\n    error\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment EventPoolInformation on EventPoolInformation {\n    fix\n    inUnused\n    inRefcnt\n    inNothing\n    missing\n  }\n"): (typeof documents)["\n  fragment EventPoolInformation on EventPoolInformation {\n    fix\n    inUnused\n    inRefcnt\n    inNothing\n    missing\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment EventPoolCleanedInformation on EventPoolCleanedInformation {\n    size\n    count\n  }\n"): (typeof documents)["\n  fragment EventPoolCleanedInformation on EventPoolCleanedInformation {\n    size\n    count\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment EventHashConversionInformation on EventHashConversionInformation {\n    count\n    algorithm\n  }\n"): (typeof documents)["\n  fragment EventHashConversionInformation on EventHashConversionInformation {\n    count\n    algorithm\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "query Hosts {\n  hosts {\n    name\n    lastBackup {\n      number\n      startDate\n      fileSize\n      completed\n      agentVersion\n    }\n    agentVersion\n    availibilityState\n    timeSinceLastBackup\n    dateToNextBackup\n    lastBackupState\n    configuration {\n      schedule {\n        activated\n      }\n    }\n  }\n}"): (typeof documents)["query Hosts {\n  hosts {\n    name\n    lastBackup {\n      number\n      startDate\n      fileSize\n      completed\n      agentVersion\n    }\n    agentVersion\n    availibilityState\n    timeSinceLastBackup\n    dateToNextBackup\n    lastBackupState\n    configuration {\n      schedule {\n        activated\n      }\n    }\n  }\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "query Backups($hostname: String!) {\n  backups(hostname: $hostname) {\n    id\n    number\n    completed\n    startDate\n    endDate\n    errorCount\n    fileCount\n    newFileCount\n    existingFileCount\n    removedFileCount\n    modifiedFileCount\n    fileSize\n    newFileSize\n    existingFileSize\n    speed\n  }\n}"): (typeof documents)["query Backups($hostname: String!) {\n  backups(hostname: $hostname) {\n    id\n    number\n    completed\n    startDate\n    endDate\n    errorCount\n    fileCount\n    newFileCount\n    existingFileCount\n    removedFileCount\n    modifiedFileCount\n    fileSize\n    newFileSize\n    existingFileSize\n    speed\n  }\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "query BackupsBrowse($hostname: String!, $number: Int!, $sharePath: String!, $path: String!) {\n  backup(hostname: $hostname, number: $number) {\n    id\n    files(sharePath: $sharePath, path: $path) {\n      ...FragmentFileDescription\n    }\n  }\n}"): (typeof documents)["query BackupsBrowse($hostname: String!, $number: Int!, $sharePath: String!, $path: String!) {\n  backup(hostname: $hostname, number: $number) {\n    id\n    files(sharePath: $sharePath, path: $path) {\n      ...FragmentFileDescription\n    }\n  }\n}"];
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
export function graphql(source: "query SharesBrowse($hostname: String!, $number: Int!) {\n  backup(hostname: $hostname, number: $number) {\n    id\n    shares {\n      ...FragmentFileDescription\n    }\n  }\n}"): (typeof documents)["query SharesBrowse($hostname: String!, $number: Int!) {\n  backup(hostname: $hostname, number: $number) {\n    id\n    shares {\n      ...FragmentFileDescription\n    }\n  }\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment JobPoolResponse on JobResponse {\n    id\n  }\n"): (typeof documents)["\n  fragment JobPoolResponse on JobResponse {\n    id\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n      mutation restoreBackup($input: RestoreInput!) {\n        restoreBackup(input: $input) {\n          ...JobPoolResponse\n        }\n      }\n    "): (typeof documents)["\n      mutation restoreBackup($input: RestoreInput!) {\n        restoreBackup(input: $input) {\n          ...JobPoolResponse\n        }\n      }\n    "];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n      mutation clearCache {\n        clearCache {\n          void\n        }\n      }\n    "): (typeof documents)["\n      mutation clearCache {\n        clearCache {\n          void\n        }\n      }\n    "];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query Events($firstEvent: DateTime!, $lastEvent: DateTime!) {\n    events(firstEvent: $firstEvent, lastEvent: $lastEvent) {\n      ...ApplicationEvent\n    }\n  }\n"): (typeof documents)["\n  query Events($firstEvent: DateTime!, $lastEvent: DateTime!) {\n    events(firstEvent: $firstEvent, lastEvent: $lastEvent) {\n      ...ApplicationEvent\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n      mutation cleanupPool {\n        cleanupPool {\n          ...JobPoolResponse\n        }\n      }\n    "): (typeof documents)["\n      mutation cleanupPool {\n        cleanupPool {\n          ...JobPoolResponse\n        }\n      }\n    "];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n      mutation fsckPool($fix: Boolean!) {\n        checkAndFixPool(fix: $fix) {\n          ...JobPoolResponse\n        }\n      }\n    "): (typeof documents)["\n      mutation fsckPool($fix: Boolean!) {\n        checkAndFixPool(fix: $fix) {\n          ...JobPoolResponse\n        }\n      }\n    "];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n      mutation verifyChecksum {\n        verifyChecksum {\n          ...JobPoolResponse\n        }\n      }\n    "): (typeof documents)["\n      mutation verifyChecksum {\n        verifyChecksum {\n          ...JobPoolResponse\n        }\n      }\n    "];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n      query ServerInformations {\n        informations {\n          platform\n          uptime\n          hostname\n          woodstockVersion\n        }\n      }\n    "): (typeof documents)["\n      query ServerInformations {\n        informations {\n          platform\n          uptime\n          hostname\n          woodstockVersion\n        }\n      }\n    "];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n      query DiskUsageStatistics {\n        statistics {\n          hosts {\n            host\n\n            size\n            compressedSize\n          }\n        }\n      }\n    "): (typeof documents)["\n      query DiskUsageStatistics {\n        statistics {\n          hosts {\n            host\n\n            size\n            compressedSize\n          }\n        }\n      }\n    "];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n      query QueueStatistics {\n        queueStats {\n          active\n          waiting\n          failed\n          delayed\n          completed\n        }\n      }\n    "): (typeof documents)["\n      query QueueStatistics {\n        queueStats {\n          active\n          waiting\n          failed\n          delayed\n          completed\n        }\n      }\n    "];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n      query PoolStatistics {\n        statistics {\n          diskUsage {\n            used\n            usedLastMonth\n            free\n            total\n          }\n          poolUsage {\n            nbChunk\n            nbChunkLastMonth\n            nbChunkRange {\n              time\n              value\n            }\n\n            nbRef\n            nbRefLastMonth\n\n            size\n            compressedSize\n            compressedSizeLastMonth\n            compressedSizeRange {\n              time\n              value\n            }\n\n            unusedSize\n          }\n        }\n      }\n    "): (typeof documents)["\n      query PoolStatistics {\n        statistics {\n          diskUsage {\n            used\n            usedLastMonth\n            free\n            total\n          }\n          poolUsage {\n            nbChunk\n            nbChunkLastMonth\n            nbChunkRange {\n              time\n              value\n            }\n\n            nbRef\n            nbRefLastMonth\n\n            size\n            compressedSize\n            compressedSizeLastMonth\n            compressedSizeRange {\n              time\n              value\n            }\n\n            unusedSize\n          }\n        }\n      }\n    "];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment ProgressTask on JobProgression {\n    progressCurrent\n    progressMax\n\n    fileSize\n    newFileSize\n\n    compressedFileSize\n    newCompressedFileSize\n\n    fileCount\n    newFileCount\n\n    errorCount\n\n    percent\n    speed\n  }\n"): (typeof documents)["\n  fragment ProgressTask on JobProgression {\n    progressCurrent\n    progressMax\n\n    fileSize\n    newFileSize\n\n    compressedFileSize\n    newCompressedFileSize\n\n    fileCount\n    newFileCount\n\n    errorCount\n\n    percent\n    speed\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment TaskDescription on SubTaskOrGroupTasks {\n    __typename\n    ... on JobSubTask {\n      taskName\n      description\n      state\n    }\n  }\n"): (typeof documents)["\n  fragment TaskDescription on SubTaskOrGroupTasks {\n    __typename\n    ... on JobSubTask {\n      taskName\n      description\n      state\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment BackupTask on SubTaskOrGroupTasks {\n    __typename\n    ... on JobGroupTasks {\n      groupName\n      description\n      state\n      progression {\n        ...ProgressTask\n      }\n      taskDescription: subtasks {\n        ...TaskDescription\n      }\n    }\n    ... on JobSubTask {\n      taskName\n      description\n      state\n      progression {\n        ...ProgressTask\n      }\n    }\n  }\n"): (typeof documents)["\n  fragment BackupTask on SubTaskOrGroupTasks {\n    __typename\n    ... on JobGroupTasks {\n      groupName\n      description\n      state\n      progression {\n        ...ProgressTask\n      }\n      taskDescription: subtasks {\n        ...TaskDescription\n      }\n    }\n    ... on JobSubTask {\n      taskName\n      description\n      state\n      progression {\n        ...ProgressTask\n      }\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment Job on Job {\n    id\n    queueName\n    name\n    failedReason\n    state\n    data {\n      host\n      number\n      startDate\n      groupName\n      description\n      ip\n      state\n      progression {\n        ...ProgressTask\n      }\n      subtasks {\n        ...BackupTask\n      }\n    }\n  }\n"): (typeof documents)["\n  fragment Job on Job {\n    id\n    queueName\n    name\n    failedReason\n    state\n    data {\n      host\n      number\n      startDate\n      groupName\n      description\n      ip\n      state\n      progression {\n        ...ProgressTask\n      }\n      subtasks {\n        ...BackupTask\n      }\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n      query Tasks($input: QueueListInput!) {\n        queue(input: $input) {\n          ...Job\n        }\n      }\n    "): (typeof documents)["\n      query Tasks($input: QueueListInput!) {\n        queue(input: $input) {\n          ...Job\n        }\n      }\n    "];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n      subscription QueueTasksJobUpdated {\n        jobUpdated {\n          ...Job\n        }\n      }\n    "): (typeof documents)["\n      subscription QueueTasksJobUpdated {\n        jobUpdated {\n          ...Job\n        }\n      }\n    "];

export function graphql(source: string) {
  return (documents as any)[source] ?? {};
}

export type DocumentType<TDocumentNode extends DocumentNode<any, any>> = TDocumentNode extends DocumentNode<  infer TType,  any>  ? TType  : never;