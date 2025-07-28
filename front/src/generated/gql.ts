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
type Documents = {
    "\n  fragment ApplicationEvent on ApplicationEvent {\n    uuid\n    type\n    step\n    source\n    timestamp\n    errorMessages\n    status\n    information {\n      __typename\n      ... on EventBackupInformation {\n        ...EventBackupInformation\n      }\n      ... on EventPoolInformation {\n        ...EventPoolInformation\n      }\n      ... on EventPoolCleanedInformation {\n        ...EventPoolCleanedInformation\n      }\n      ... on EventHashConversionInformation {\n        ...EventHashConversionInformation\n      }\n    }\n  }\n": typeof types.ApplicationEventFragmentDoc,
    "\n  fragment EventBackupInformation on EventBackupInformation {\n    hostname\n    number\n    sharePath\n  }\n": typeof types.EventBackupInformationFragmentDoc,
    "\n  fragment EventPoolInformation on EventPoolInformation {\n    fix\n    refcount\n    refcountError\n    inUnused\n    inRefcnt\n    inNothing\n    missing\n    chunkCount\n    chunkError\n  }\n": typeof types.EventPoolInformationFragmentDoc,
    "\n  fragment EventPoolCleanedInformation on EventPoolCleanedInformation {\n    size\n    count\n  }\n": typeof types.EventPoolCleanedInformationFragmentDoc,
    "\n  fragment EventHashConversionInformation on EventHashConversionInformation {\n    count\n    algorithm\n  }\n": typeof types.EventHashConversionInformationFragmentDoc,
    "\n  query PoolHealth {\n    poolHealth {\n      healthy\n      isDirty\n      pendingCount\n    }\n  }\n": typeof types.PoolHealthDocument,
    "query Host($hostname: String!) {\n  host(hostname: $hostname) {\n    name\n    agentVersion\n    lastBackup {\n      agentVersion\n      status {\n        ...BackupStatusFields\n      }\n    }\n    availibilityState\n    timeSinceLastBackup\n    dateToNextBackup\n    addresses\n    configuration {\n      operations {\n        preCommands {\n          command\n        }\n        operation {\n          shares {\n            name\n          }\n        }\n        postCommands {\n          command\n        }\n      }\n      schedule {\n        activated\n      }\n    }\n  }\n}": typeof types.HostDocument,
    "query Hosts {\n  hosts {\n    name\n    lastBackup {\n      number\n      startDate\n      fileSize\n      status {\n        ...BackupStatusFields\n      }\n      agentVersion\n    }\n    agentVersion\n    availibilityState\n    timeSinceLastBackup\n    dateToNextBackup\n    configuration {\n      schedule {\n        activated\n      }\n    }\n  }\n}": typeof types.HostsDocument,
    "query Backup($hostname: String!, $id: String!) {\n  backup(hostname: $hostname, id: $id) {\n    id\n    number\n    status {\n      ...BackupStatusFields\n    }\n    startDate\n    endDate\n    errorCount\n    fileCount\n    newFileCount\n    existingFileCount\n    removedFileCount\n    modifiedFileCount\n    fileSize\n    newFileSize\n    existingFileSize\n    speed\n  }\n}": typeof types.BackupDocument,
    "query Backups($hostname: String!) {\n  backups(hostname: $hostname) {\n    id\n    number\n    status {\n      ...BackupStatusFields\n    }\n    startDate\n    endDate\n    errorCount\n    fileCount\n    newFileCount\n    existingFileCount\n    removedFileCount\n    modifiedFileCount\n    fileSize\n    newFileSize\n    existingFileSize\n    speed\n  }\n}": typeof types.BackupsDocument,
    "query BackupsBrowse($hostname: String!, $id: String!, $sharePath: String!, $path: Buffer!) {\n  backup(hostname: $hostname, id: $id) {\n    id\n    files(sharePath: $sharePath, path: $path) {\n      ...FragmentFileDescription\n    }\n  }\n}": typeof types.BackupsBrowseDocument,
    "mutation createBackup($hostname: String!) {\n  createBackup(hostname: $hostname) {\n    id\n  }\n}": typeof types.CreateBackupDocument,
    "mutation removeBackup($hostname: String!, $id: String!) {\n  removeBackup(hostname: $hostname, id: $id) {\n    id\n  }\n}": typeof types.RemoveBackupDocument,
    "fragment BackupStatusFields on BackupStatusDto {\n  statusType\n  finishingStage\n  abortingStage\n  failedStage\n  removingStage\n}": typeof types.BackupStatusFieldsFragmentDoc,
    "fragment FragmentFileDescription on FileDescription {\n  path\n  type\n  stats {\n    ownerId\n    groupId\n    mode\n    size\n    lastModified\n  }\n  symlink\n}": typeof types.FragmentFileDescriptionFragmentDoc,
    "query SharesBrowse($hostname: String!, $id: String!) {\n  backup(hostname: $hostname, id: $id) {\n    id\n    shares {\n      ...FragmentFileDescription\n    }\n  }\n}": typeof types.SharesBrowseDocument,
    "query PoolHealth {\n  poolHealth {\n    healthy\n    isDirty\n    pendingCount\n  }\n}": typeof types.PoolHealthDocument,
    "\n  subscription BackupUpdated($hostname: String!) {\n    backupUpdated(hostname: $hostname) {\n      id\n      number\n      status {\n        statusType\n        finishingStage\n        abortingStage\n        failedStage\n        removingStage\n      }\n      startDate\n      endDate\n      errorCount\n      fileCount\n      newFileCount\n      existingFileCount\n      removedFileCount\n      modifiedFileCount\n      fileSize\n      newFileSize\n      existingFileSize\n      speed\n    }\n  }\n": typeof types.BackupUpdatedDocument,
    "\n  subscription JobRemoveUpdated($host: String, $kind: String) {\n    jobUpdated(host: $host, kind: $kind) {\n      jobId\n      status\n    }\n  }\n": typeof types.JobRemoveUpdatedDocument,
    "\n  subscription JobBackupUpdated($host: String, $kind: String) {\n    jobUpdated(host: $host, kind: $kind) {\n      jobId\n      status\n    }\n  }\n": typeof types.JobBackupUpdatedDocument,
    "\n  fragment JobPoolResponse on JobResponse {\n    id\n  }\n": typeof types.JobPoolResponseFragmentDoc,
    "\n      mutation restoreBackup($input: RestoreInput!) {\n        restoreBackup(input: $input) {\n          ...JobPoolResponse\n        }\n      }\n    ": typeof types.RestoreBackupDocument,
    "\n      mutation clearCache {\n        clearCache {\n          id\n        }\n      }\n    ": typeof types.ClearCacheDocument,
    "\n  query Events($firstEvent: DateTime!, $lastEvent: DateTime!, $limit: Int, $offset: Int) {\n    events(firstEvent: $firstEvent, lastEvent: $lastEvent, limit: $limit, offset: $offset) {\n      ...ApplicationEvent\n    }\n  }\n": typeof types.EventsDocument,
    "\n      mutation cleanupPool {\n        cleanupPool {\n          ...JobPoolResponse\n        }\n      }\n    ": typeof types.CleanupPoolDocument,
    "\n      mutation checkAndFixPool($fix: Boolean!, $verifyChunks: Boolean!) {\n        checkAndFixPool(fix: $fix, verifyChunks: $verifyChunks) {\n          ...JobPoolResponse\n        }\n      }\n    ": typeof types.CheckAndFixPoolDocument,
    "\n      query ServerInformations {\n        informations {\n          uptime\n          hostname\n          woodstockVersion\n        }\n      }\n    ": typeof types.ServerInformationsDocument,
    "\n      query DiskUsageStatistics {\n        statistics {\n          hosts {\n            host\n\n            size\n            compressedSize\n          }\n        }\n      }\n    ": typeof types.DiskUsageStatisticsDocument,
    "\n      query QueueStatistics {\n        queueStats {\n          pending\n          running\n          success\n          failed\n          dead\n        }\n      }\n    ": typeof types.QueueStatisticsDocument,
    "\n      query PoolStatistics {\n        statistics {\n          diskUsage {\n            used\n            usedLastMonth\n            free\n            total\n          }\n          poolUsage {\n            nbChunk\n            nbChunkLastMonth\n            nbChunkRange {\n              time\n              value\n            }\n\n            nbRef\n            nbRefLastMonth\n\n            size\n            compressedSize\n            compressedSizeLastMonth\n            compressedSizeRange {\n              time\n              value\n            }\n\n            unusedSize\n          }\n        }\n      }\n    ": typeof types.PoolStatisticsDocument,
    "\n  fragment JobBackupData on JobBackupData {\n    host\n    number\n    ip\n    startDate\n  }\n": typeof types.JobBackupDataFragmentDoc,
    "\n  fragment JobRestoreData on JobRestoreData {\n    host\n    number\n    ip\n    startDate\n    destinationDirectory\n    files {\n      share\n      selection\n    }\n  }\n": typeof types.JobRestoreDataFragmentDoc,
    "\n  fragment JobRemoveData on JobRemoveData {\n    host\n    number\n    startDate\n  }\n": typeof types.JobRemoveDataFragmentDoc,
    "\n  fragment JobCleanupData on JobCleanupData {\n    target\n  }\n": typeof types.JobCleanupDataFragmentDoc,
    "\n  fragment JobFsckData on JobFsckData {\n    dryRun\n    verifyChunks\n  }\n": typeof types.JobFsckDataFragmentDoc,
    "\n  fragment BackupTaskState on JobBackupTaskState {\n    backupExecutionState: executionState\n    backupErrorState: errorState\n    backupErrorMessage: errorMessage\n    globalProgression: progression {\n      startDate\n      startTransferDate\n      endTransferDate\n      fileSize\n      newFileSize\n      modifiedFileSize\n      compressedFileSize\n      newCompressedFileSize\n      modifiedCompressedFileSize\n      fileCount\n      newFileCount\n      modifiedFileCount\n      removedFileCount\n      errorCount\n      speed\n      percent\n      progressCurrent\n      progressMax\n    }\n    preCommandStates {\n      executionState\n      command {\n        command\n      }\n    }\n    shareStates {\n      share\n      executionState\n      backupProgression {\n        startDate\n        startTransferDate\n        endTransferDate\n        fileSize\n        newFileSize\n        modifiedFileSize\n        compressedFileSize\n        newCompressedFileSize\n        modifiedCompressedFileSize\n        fileCount\n        newFileCount\n        modifiedFileCount\n        removedFileCount\n        errorCount\n        speed\n        percent\n        progressCurrent\n        progressMax\n      }\n      fileListProgression {\n        fileSize\n        newFileSize\n        modifiedFileSize\n        newFileCount\n        modifiedFileCount\n        removedFileCount\n      }\n    }\n    postCommandStates {\n      executionState\n      command {\n        command\n      }\n    }\n  }\n": typeof types.BackupTaskStateFragmentDoc,
    "\n  fragment RestoreTaskState on JobRestoreTaskState {\n    restoreExecutionState: executionState\n    restoreErrorState: errorState\n    restoreErrorMessage: errorMessage\n    restoreProgression: globalProgression {\n      startDate\n      startTransferDate\n      endTransferDate\n      fileSize\n      newFileSize\n      modifiedFileSize\n      compressedFileSize\n      newCompressedFileSize\n      modifiedCompressedFileSize\n      fileCount\n      newFileCount\n      modifiedFileCount\n      removedFileCount\n      errorCount\n      speed\n      percent\n      progressCurrent\n      progressMax\n    }\n  }\n": typeof types.RestoreTaskStateFragmentDoc,
    "\n  fragment RemoveTaskState on JobRemoveState {\n    removeExecutionState: executionState\n    removeErrorState: errorState\n    removeErrorMessage: errorMessage\n  }\n": typeof types.RemoveTaskStateFragmentDoc,
    "\n  fragment CleanerTaskState on JobCleanerTaskState {\n    cleanerExecutionState: executionState\n    cleanerErrorState: errorState\n    cleanerErrorMessage: errorMessage\n    cleanerProgress: progression {\n      progressMax\n      progressCurrent\n      fileSize\n      compressedFileSize\n    }\n  }\n": typeof types.CleanerTaskStateFragmentDoc,
    "\n  fragment FsckTaskState on JobFsckTaskState {\n    fsckExecutionState: executionState\n    fsckErrorState: errorState\n    fsckErrorMessage: errorMessage\n    dryRun\n    refcntProgression {\n      progressMax\n      progressCurrent\n      errorCount\n      totalCount\n    }\n    unusedProgression {\n      progressMax\n      progressCurrent\n      inNothing\n      inRefcnt\n      inUnused\n      missing\n    }\n    chunkProgression {\n      progressMax\n      progressCurrent\n      errorCount\n      totalCount\n    }\n  }\n": typeof types.FsckTaskStateFragmentDoc,
    "\n  fragment Job on Job {\n    jobId\n    kind\n    status\n    timestamp\n    host\n    failedReason\n    data {\n      ...JobBackupData\n      ...JobRestoreData\n      ...JobRemoveData\n      ...JobCleanupData\n      ...JobFsckData\n    }\n    progress {\n      ...BackupTaskState\n      ...RestoreTaskState\n      ...RemoveTaskState\n      ...CleanerTaskState\n      ...FsckTaskState\n    }\n  }\n": typeof types.JobFragmentDoc,
    "\n      query Tasks($input: QueueListInput!) {\n        queue(input: $input) {\n          ...Job\n        }\n      }\n    ": typeof types.TasksDocument,
    "\n      subscription QueueTasksJobUpdated {\n        jobUpdated {\n          ...Job\n        }\n      }\n    ": typeof types.QueueTasksJobUpdatedDocument,
};
const documents: Documents = {
    "\n  fragment ApplicationEvent on ApplicationEvent {\n    uuid\n    type\n    step\n    source\n    timestamp\n    errorMessages\n    status\n    information {\n      __typename\n      ... on EventBackupInformation {\n        ...EventBackupInformation\n      }\n      ... on EventPoolInformation {\n        ...EventPoolInformation\n      }\n      ... on EventPoolCleanedInformation {\n        ...EventPoolCleanedInformation\n      }\n      ... on EventHashConversionInformation {\n        ...EventHashConversionInformation\n      }\n    }\n  }\n": types.ApplicationEventFragmentDoc,
    "\n  fragment EventBackupInformation on EventBackupInformation {\n    hostname\n    number\n    sharePath\n  }\n": types.EventBackupInformationFragmentDoc,
    "\n  fragment EventPoolInformation on EventPoolInformation {\n    fix\n    refcount\n    refcountError\n    inUnused\n    inRefcnt\n    inNothing\n    missing\n    chunkCount\n    chunkError\n  }\n": types.EventPoolInformationFragmentDoc,
    "\n  fragment EventPoolCleanedInformation on EventPoolCleanedInformation {\n    size\n    count\n  }\n": types.EventPoolCleanedInformationFragmentDoc,
    "\n  fragment EventHashConversionInformation on EventHashConversionInformation {\n    count\n    algorithm\n  }\n": types.EventHashConversionInformationFragmentDoc,
    "\n  query PoolHealth {\n    poolHealth {\n      healthy\n      isDirty\n      pendingCount\n    }\n  }\n": types.PoolHealthDocument,
    "query Host($hostname: String!) {\n  host(hostname: $hostname) {\n    name\n    agentVersion\n    lastBackup {\n      agentVersion\n      status {\n        ...BackupStatusFields\n      }\n    }\n    availibilityState\n    timeSinceLastBackup\n    dateToNextBackup\n    addresses\n    configuration {\n      operations {\n        preCommands {\n          command\n        }\n        operation {\n          shares {\n            name\n          }\n        }\n        postCommands {\n          command\n        }\n      }\n      schedule {\n        activated\n      }\n    }\n  }\n}": types.HostDocument,
    "query Hosts {\n  hosts {\n    name\n    lastBackup {\n      number\n      startDate\n      fileSize\n      status {\n        ...BackupStatusFields\n      }\n      agentVersion\n    }\n    agentVersion\n    availibilityState\n    timeSinceLastBackup\n    dateToNextBackup\n    configuration {\n      schedule {\n        activated\n      }\n    }\n  }\n}": types.HostsDocument,
    "query Backup($hostname: String!, $id: String!) {\n  backup(hostname: $hostname, id: $id) {\n    id\n    number\n    status {\n      ...BackupStatusFields\n    }\n    startDate\n    endDate\n    errorCount\n    fileCount\n    newFileCount\n    existingFileCount\n    removedFileCount\n    modifiedFileCount\n    fileSize\n    newFileSize\n    existingFileSize\n    speed\n  }\n}": types.BackupDocument,
    "query Backups($hostname: String!) {\n  backups(hostname: $hostname) {\n    id\n    number\n    status {\n      ...BackupStatusFields\n    }\n    startDate\n    endDate\n    errorCount\n    fileCount\n    newFileCount\n    existingFileCount\n    removedFileCount\n    modifiedFileCount\n    fileSize\n    newFileSize\n    existingFileSize\n    speed\n  }\n}": types.BackupsDocument,
    "query BackupsBrowse($hostname: String!, $id: String!, $sharePath: String!, $path: Buffer!) {\n  backup(hostname: $hostname, id: $id) {\n    id\n    files(sharePath: $sharePath, path: $path) {\n      ...FragmentFileDescription\n    }\n  }\n}": types.BackupsBrowseDocument,
    "mutation createBackup($hostname: String!) {\n  createBackup(hostname: $hostname) {\n    id\n  }\n}": types.CreateBackupDocument,
    "mutation removeBackup($hostname: String!, $id: String!) {\n  removeBackup(hostname: $hostname, id: $id) {\n    id\n  }\n}": types.RemoveBackupDocument,
    "fragment BackupStatusFields on BackupStatusDto {\n  statusType\n  finishingStage\n  abortingStage\n  failedStage\n  removingStage\n}": types.BackupStatusFieldsFragmentDoc,
    "fragment FragmentFileDescription on FileDescription {\n  path\n  type\n  stats {\n    ownerId\n    groupId\n    mode\n    size\n    lastModified\n  }\n  symlink\n}": types.FragmentFileDescriptionFragmentDoc,
    "query SharesBrowse($hostname: String!, $id: String!) {\n  backup(hostname: $hostname, id: $id) {\n    id\n    shares {\n      ...FragmentFileDescription\n    }\n  }\n}": types.SharesBrowseDocument,
    "query PoolHealth {\n  poolHealth {\n    healthy\n    isDirty\n    pendingCount\n  }\n}": types.PoolHealthDocument,
    "\n  subscription BackupUpdated($hostname: String!) {\n    backupUpdated(hostname: $hostname) {\n      id\n      number\n      status {\n        statusType\n        finishingStage\n        abortingStage\n        failedStage\n        removingStage\n      }\n      startDate\n      endDate\n      errorCount\n      fileCount\n      newFileCount\n      existingFileCount\n      removedFileCount\n      modifiedFileCount\n      fileSize\n      newFileSize\n      existingFileSize\n      speed\n    }\n  }\n": types.BackupUpdatedDocument,
    "\n  subscription JobRemoveUpdated($host: String, $kind: String) {\n    jobUpdated(host: $host, kind: $kind) {\n      jobId\n      status\n    }\n  }\n": types.JobRemoveUpdatedDocument,
    "\n  subscription JobBackupUpdated($host: String, $kind: String) {\n    jobUpdated(host: $host, kind: $kind) {\n      jobId\n      status\n    }\n  }\n": types.JobBackupUpdatedDocument,
    "\n  fragment JobPoolResponse on JobResponse {\n    id\n  }\n": types.JobPoolResponseFragmentDoc,
    "\n      mutation restoreBackup($input: RestoreInput!) {\n        restoreBackup(input: $input) {\n          ...JobPoolResponse\n        }\n      }\n    ": types.RestoreBackupDocument,
    "\n      mutation clearCache {\n        clearCache {\n          id\n        }\n      }\n    ": types.ClearCacheDocument,
    "\n  query Events($firstEvent: DateTime!, $lastEvent: DateTime!, $limit: Int, $offset: Int) {\n    events(firstEvent: $firstEvent, lastEvent: $lastEvent, limit: $limit, offset: $offset) {\n      ...ApplicationEvent\n    }\n  }\n": types.EventsDocument,
    "\n      mutation cleanupPool {\n        cleanupPool {\n          ...JobPoolResponse\n        }\n      }\n    ": types.CleanupPoolDocument,
    "\n      mutation checkAndFixPool($fix: Boolean!, $verifyChunks: Boolean!) {\n        checkAndFixPool(fix: $fix, verifyChunks: $verifyChunks) {\n          ...JobPoolResponse\n        }\n      }\n    ": types.CheckAndFixPoolDocument,
    "\n      query ServerInformations {\n        informations {\n          uptime\n          hostname\n          woodstockVersion\n        }\n      }\n    ": types.ServerInformationsDocument,
    "\n      query DiskUsageStatistics {\n        statistics {\n          hosts {\n            host\n\n            size\n            compressedSize\n          }\n        }\n      }\n    ": types.DiskUsageStatisticsDocument,
    "\n      query QueueStatistics {\n        queueStats {\n          pending\n          running\n          success\n          failed\n          dead\n        }\n      }\n    ": types.QueueStatisticsDocument,
    "\n      query PoolStatistics {\n        statistics {\n          diskUsage {\n            used\n            usedLastMonth\n            free\n            total\n          }\n          poolUsage {\n            nbChunk\n            nbChunkLastMonth\n            nbChunkRange {\n              time\n              value\n            }\n\n            nbRef\n            nbRefLastMonth\n\n            size\n            compressedSize\n            compressedSizeLastMonth\n            compressedSizeRange {\n              time\n              value\n            }\n\n            unusedSize\n          }\n        }\n      }\n    ": types.PoolStatisticsDocument,
    "\n  fragment JobBackupData on JobBackupData {\n    host\n    number\n    ip\n    startDate\n  }\n": types.JobBackupDataFragmentDoc,
    "\n  fragment JobRestoreData on JobRestoreData {\n    host\n    number\n    ip\n    startDate\n    destinationDirectory\n    files {\n      share\n      selection\n    }\n  }\n": types.JobRestoreDataFragmentDoc,
    "\n  fragment JobRemoveData on JobRemoveData {\n    host\n    number\n    startDate\n  }\n": types.JobRemoveDataFragmentDoc,
    "\n  fragment JobCleanupData on JobCleanupData {\n    target\n  }\n": types.JobCleanupDataFragmentDoc,
    "\n  fragment JobFsckData on JobFsckData {\n    dryRun\n    verifyChunks\n  }\n": types.JobFsckDataFragmentDoc,
    "\n  fragment BackupTaskState on JobBackupTaskState {\n    backupExecutionState: executionState\n    backupErrorState: errorState\n    backupErrorMessage: errorMessage\n    globalProgression: progression {\n      startDate\n      startTransferDate\n      endTransferDate\n      fileSize\n      newFileSize\n      modifiedFileSize\n      compressedFileSize\n      newCompressedFileSize\n      modifiedCompressedFileSize\n      fileCount\n      newFileCount\n      modifiedFileCount\n      removedFileCount\n      errorCount\n      speed\n      percent\n      progressCurrent\n      progressMax\n    }\n    preCommandStates {\n      executionState\n      command {\n        command\n      }\n    }\n    shareStates {\n      share\n      executionState\n      backupProgression {\n        startDate\n        startTransferDate\n        endTransferDate\n        fileSize\n        newFileSize\n        modifiedFileSize\n        compressedFileSize\n        newCompressedFileSize\n        modifiedCompressedFileSize\n        fileCount\n        newFileCount\n        modifiedFileCount\n        removedFileCount\n        errorCount\n        speed\n        percent\n        progressCurrent\n        progressMax\n      }\n      fileListProgression {\n        fileSize\n        newFileSize\n        modifiedFileSize\n        newFileCount\n        modifiedFileCount\n        removedFileCount\n      }\n    }\n    postCommandStates {\n      executionState\n      command {\n        command\n      }\n    }\n  }\n": types.BackupTaskStateFragmentDoc,
    "\n  fragment RestoreTaskState on JobRestoreTaskState {\n    restoreExecutionState: executionState\n    restoreErrorState: errorState\n    restoreErrorMessage: errorMessage\n    restoreProgression: globalProgression {\n      startDate\n      startTransferDate\n      endTransferDate\n      fileSize\n      newFileSize\n      modifiedFileSize\n      compressedFileSize\n      newCompressedFileSize\n      modifiedCompressedFileSize\n      fileCount\n      newFileCount\n      modifiedFileCount\n      removedFileCount\n      errorCount\n      speed\n      percent\n      progressCurrent\n      progressMax\n    }\n  }\n": types.RestoreTaskStateFragmentDoc,
    "\n  fragment RemoveTaskState on JobRemoveState {\n    removeExecutionState: executionState\n    removeErrorState: errorState\n    removeErrorMessage: errorMessage\n  }\n": types.RemoveTaskStateFragmentDoc,
    "\n  fragment CleanerTaskState on JobCleanerTaskState {\n    cleanerExecutionState: executionState\n    cleanerErrorState: errorState\n    cleanerErrorMessage: errorMessage\n    cleanerProgress: progression {\n      progressMax\n      progressCurrent\n      fileSize\n      compressedFileSize\n    }\n  }\n": types.CleanerTaskStateFragmentDoc,
    "\n  fragment FsckTaskState on JobFsckTaskState {\n    fsckExecutionState: executionState\n    fsckErrorState: errorState\n    fsckErrorMessage: errorMessage\n    dryRun\n    refcntProgression {\n      progressMax\n      progressCurrent\n      errorCount\n      totalCount\n    }\n    unusedProgression {\n      progressMax\n      progressCurrent\n      inNothing\n      inRefcnt\n      inUnused\n      missing\n    }\n    chunkProgression {\n      progressMax\n      progressCurrent\n      errorCount\n      totalCount\n    }\n  }\n": types.FsckTaskStateFragmentDoc,
    "\n  fragment Job on Job {\n    jobId\n    kind\n    status\n    timestamp\n    host\n    failedReason\n    data {\n      ...JobBackupData\n      ...JobRestoreData\n      ...JobRemoveData\n      ...JobCleanupData\n      ...JobFsckData\n    }\n    progress {\n      ...BackupTaskState\n      ...RestoreTaskState\n      ...RemoveTaskState\n      ...CleanerTaskState\n      ...FsckTaskState\n    }\n  }\n": types.JobFragmentDoc,
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
export function graphql(source: "\n  fragment ApplicationEvent on ApplicationEvent {\n    uuid\n    type\n    step\n    source\n    timestamp\n    errorMessages\n    status\n    information {\n      __typename\n      ... on EventBackupInformation {\n        ...EventBackupInformation\n      }\n      ... on EventPoolInformation {\n        ...EventPoolInformation\n      }\n      ... on EventPoolCleanedInformation {\n        ...EventPoolCleanedInformation\n      }\n      ... on EventHashConversionInformation {\n        ...EventHashConversionInformation\n      }\n    }\n  }\n"): (typeof documents)["\n  fragment ApplicationEvent on ApplicationEvent {\n    uuid\n    type\n    step\n    source\n    timestamp\n    errorMessages\n    status\n    information {\n      __typename\n      ... on EventBackupInformation {\n        ...EventBackupInformation\n      }\n      ... on EventPoolInformation {\n        ...EventPoolInformation\n      }\n      ... on EventPoolCleanedInformation {\n        ...EventPoolCleanedInformation\n      }\n      ... on EventHashConversionInformation {\n        ...EventHashConversionInformation\n      }\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment EventBackupInformation on EventBackupInformation {\n    hostname\n    number\n    sharePath\n  }\n"): (typeof documents)["\n  fragment EventBackupInformation on EventBackupInformation {\n    hostname\n    number\n    sharePath\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment EventPoolInformation on EventPoolInformation {\n    fix\n    refcount\n    refcountError\n    inUnused\n    inRefcnt\n    inNothing\n    missing\n    chunkCount\n    chunkError\n  }\n"): (typeof documents)["\n  fragment EventPoolInformation on EventPoolInformation {\n    fix\n    refcount\n    refcountError\n    inUnused\n    inRefcnt\n    inNothing\n    missing\n    chunkCount\n    chunkError\n  }\n"];
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
export function graphql(source: "\n  query PoolHealth {\n    poolHealth {\n      healthy\n      isDirty\n      pendingCount\n    }\n  }\n"): (typeof documents)["\n  query PoolHealth {\n    poolHealth {\n      healthy\n      isDirty\n      pendingCount\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "query Host($hostname: String!) {\n  host(hostname: $hostname) {\n    name\n    agentVersion\n    lastBackup {\n      agentVersion\n      status {\n        ...BackupStatusFields\n      }\n    }\n    availibilityState\n    timeSinceLastBackup\n    dateToNextBackup\n    addresses\n    configuration {\n      operations {\n        preCommands {\n          command\n        }\n        operation {\n          shares {\n            name\n          }\n        }\n        postCommands {\n          command\n        }\n      }\n      schedule {\n        activated\n      }\n    }\n  }\n}"): (typeof documents)["query Host($hostname: String!) {\n  host(hostname: $hostname) {\n    name\n    agentVersion\n    lastBackup {\n      agentVersion\n      status {\n        ...BackupStatusFields\n      }\n    }\n    availibilityState\n    timeSinceLastBackup\n    dateToNextBackup\n    addresses\n    configuration {\n      operations {\n        preCommands {\n          command\n        }\n        operation {\n          shares {\n            name\n          }\n        }\n        postCommands {\n          command\n        }\n      }\n      schedule {\n        activated\n      }\n    }\n  }\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "query Hosts {\n  hosts {\n    name\n    lastBackup {\n      number\n      startDate\n      fileSize\n      status {\n        ...BackupStatusFields\n      }\n      agentVersion\n    }\n    agentVersion\n    availibilityState\n    timeSinceLastBackup\n    dateToNextBackup\n    configuration {\n      schedule {\n        activated\n      }\n    }\n  }\n}"): (typeof documents)["query Hosts {\n  hosts {\n    name\n    lastBackup {\n      number\n      startDate\n      fileSize\n      status {\n        ...BackupStatusFields\n      }\n      agentVersion\n    }\n    agentVersion\n    availibilityState\n    timeSinceLastBackup\n    dateToNextBackup\n    configuration {\n      schedule {\n        activated\n      }\n    }\n  }\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "query Backup($hostname: String!, $id: String!) {\n  backup(hostname: $hostname, id: $id) {\n    id\n    number\n    status {\n      ...BackupStatusFields\n    }\n    startDate\n    endDate\n    errorCount\n    fileCount\n    newFileCount\n    existingFileCount\n    removedFileCount\n    modifiedFileCount\n    fileSize\n    newFileSize\n    existingFileSize\n    speed\n  }\n}"): (typeof documents)["query Backup($hostname: String!, $id: String!) {\n  backup(hostname: $hostname, id: $id) {\n    id\n    number\n    status {\n      ...BackupStatusFields\n    }\n    startDate\n    endDate\n    errorCount\n    fileCount\n    newFileCount\n    existingFileCount\n    removedFileCount\n    modifiedFileCount\n    fileSize\n    newFileSize\n    existingFileSize\n    speed\n  }\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "query Backups($hostname: String!) {\n  backups(hostname: $hostname) {\n    id\n    number\n    status {\n      ...BackupStatusFields\n    }\n    startDate\n    endDate\n    errorCount\n    fileCount\n    newFileCount\n    existingFileCount\n    removedFileCount\n    modifiedFileCount\n    fileSize\n    newFileSize\n    existingFileSize\n    speed\n  }\n}"): (typeof documents)["query Backups($hostname: String!) {\n  backups(hostname: $hostname) {\n    id\n    number\n    status {\n      ...BackupStatusFields\n    }\n    startDate\n    endDate\n    errorCount\n    fileCount\n    newFileCount\n    existingFileCount\n    removedFileCount\n    modifiedFileCount\n    fileSize\n    newFileSize\n    existingFileSize\n    speed\n  }\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "query BackupsBrowse($hostname: String!, $id: String!, $sharePath: String!, $path: Buffer!) {\n  backup(hostname: $hostname, id: $id) {\n    id\n    files(sharePath: $sharePath, path: $path) {\n      ...FragmentFileDescription\n    }\n  }\n}"): (typeof documents)["query BackupsBrowse($hostname: String!, $id: String!, $sharePath: String!, $path: Buffer!) {\n  backup(hostname: $hostname, id: $id) {\n    id\n    files(sharePath: $sharePath, path: $path) {\n      ...FragmentFileDescription\n    }\n  }\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "mutation createBackup($hostname: String!) {\n  createBackup(hostname: $hostname) {\n    id\n  }\n}"): (typeof documents)["mutation createBackup($hostname: String!) {\n  createBackup(hostname: $hostname) {\n    id\n  }\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "mutation removeBackup($hostname: String!, $id: String!) {\n  removeBackup(hostname: $hostname, id: $id) {\n    id\n  }\n}"): (typeof documents)["mutation removeBackup($hostname: String!, $id: String!) {\n  removeBackup(hostname: $hostname, id: $id) {\n    id\n  }\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "fragment BackupStatusFields on BackupStatusDto {\n  statusType\n  finishingStage\n  abortingStage\n  failedStage\n  removingStage\n}"): (typeof documents)["fragment BackupStatusFields on BackupStatusDto {\n  statusType\n  finishingStage\n  abortingStage\n  failedStage\n  removingStage\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "fragment FragmentFileDescription on FileDescription {\n  path\n  type\n  stats {\n    ownerId\n    groupId\n    mode\n    size\n    lastModified\n  }\n  symlink\n}"): (typeof documents)["fragment FragmentFileDescription on FileDescription {\n  path\n  type\n  stats {\n    ownerId\n    groupId\n    mode\n    size\n    lastModified\n  }\n  symlink\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "query SharesBrowse($hostname: String!, $id: String!) {\n  backup(hostname: $hostname, id: $id) {\n    id\n    shares {\n      ...FragmentFileDescription\n    }\n  }\n}"): (typeof documents)["query SharesBrowse($hostname: String!, $id: String!) {\n  backup(hostname: $hostname, id: $id) {\n    id\n    shares {\n      ...FragmentFileDescription\n    }\n  }\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "query PoolHealth {\n  poolHealth {\n    healthy\n    isDirty\n    pendingCount\n  }\n}"): (typeof documents)["query PoolHealth {\n  poolHealth {\n    healthy\n    isDirty\n    pendingCount\n  }\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  subscription BackupUpdated($hostname: String!) {\n    backupUpdated(hostname: $hostname) {\n      id\n      number\n      status {\n        statusType\n        finishingStage\n        abortingStage\n        failedStage\n        removingStage\n      }\n      startDate\n      endDate\n      errorCount\n      fileCount\n      newFileCount\n      existingFileCount\n      removedFileCount\n      modifiedFileCount\n      fileSize\n      newFileSize\n      existingFileSize\n      speed\n    }\n  }\n"): (typeof documents)["\n  subscription BackupUpdated($hostname: String!) {\n    backupUpdated(hostname: $hostname) {\n      id\n      number\n      status {\n        statusType\n        finishingStage\n        abortingStage\n        failedStage\n        removingStage\n      }\n      startDate\n      endDate\n      errorCount\n      fileCount\n      newFileCount\n      existingFileCount\n      removedFileCount\n      modifiedFileCount\n      fileSize\n      newFileSize\n      existingFileSize\n      speed\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  subscription JobRemoveUpdated($host: String, $kind: String) {\n    jobUpdated(host: $host, kind: $kind) {\n      jobId\n      status\n    }\n  }\n"): (typeof documents)["\n  subscription JobRemoveUpdated($host: String, $kind: String) {\n    jobUpdated(host: $host, kind: $kind) {\n      jobId\n      status\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  subscription JobBackupUpdated($host: String, $kind: String) {\n    jobUpdated(host: $host, kind: $kind) {\n      jobId\n      status\n    }\n  }\n"): (typeof documents)["\n  subscription JobBackupUpdated($host: String, $kind: String) {\n    jobUpdated(host: $host, kind: $kind) {\n      jobId\n      status\n    }\n  }\n"];
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
export function graphql(source: "\n      mutation clearCache {\n        clearCache {\n          id\n        }\n      }\n    "): (typeof documents)["\n      mutation clearCache {\n        clearCache {\n          id\n        }\n      }\n    "];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query Events($firstEvent: DateTime!, $lastEvent: DateTime!, $limit: Int, $offset: Int) {\n    events(firstEvent: $firstEvent, lastEvent: $lastEvent, limit: $limit, offset: $offset) {\n      ...ApplicationEvent\n    }\n  }\n"): (typeof documents)["\n  query Events($firstEvent: DateTime!, $lastEvent: DateTime!, $limit: Int, $offset: Int) {\n    events(firstEvent: $firstEvent, lastEvent: $lastEvent, limit: $limit, offset: $offset) {\n      ...ApplicationEvent\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n      mutation cleanupPool {\n        cleanupPool {\n          ...JobPoolResponse\n        }\n      }\n    "): (typeof documents)["\n      mutation cleanupPool {\n        cleanupPool {\n          ...JobPoolResponse\n        }\n      }\n    "];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n      mutation checkAndFixPool($fix: Boolean!, $verifyChunks: Boolean!) {\n        checkAndFixPool(fix: $fix, verifyChunks: $verifyChunks) {\n          ...JobPoolResponse\n        }\n      }\n    "): (typeof documents)["\n      mutation checkAndFixPool($fix: Boolean!, $verifyChunks: Boolean!) {\n        checkAndFixPool(fix: $fix, verifyChunks: $verifyChunks) {\n          ...JobPoolResponse\n        }\n      }\n    "];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n      query ServerInformations {\n        informations {\n          uptime\n          hostname\n          woodstockVersion\n        }\n      }\n    "): (typeof documents)["\n      query ServerInformations {\n        informations {\n          uptime\n          hostname\n          woodstockVersion\n        }\n      }\n    "];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n      query DiskUsageStatistics {\n        statistics {\n          hosts {\n            host\n\n            size\n            compressedSize\n          }\n        }\n      }\n    "): (typeof documents)["\n      query DiskUsageStatistics {\n        statistics {\n          hosts {\n            host\n\n            size\n            compressedSize\n          }\n        }\n      }\n    "];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n      query QueueStatistics {\n        queueStats {\n          pending\n          running\n          success\n          failed\n          dead\n        }\n      }\n    "): (typeof documents)["\n      query QueueStatistics {\n        queueStats {\n          pending\n          running\n          success\n          failed\n          dead\n        }\n      }\n    "];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n      query PoolStatistics {\n        statistics {\n          diskUsage {\n            used\n            usedLastMonth\n            free\n            total\n          }\n          poolUsage {\n            nbChunk\n            nbChunkLastMonth\n            nbChunkRange {\n              time\n              value\n            }\n\n            nbRef\n            nbRefLastMonth\n\n            size\n            compressedSize\n            compressedSizeLastMonth\n            compressedSizeRange {\n              time\n              value\n            }\n\n            unusedSize\n          }\n        }\n      }\n    "): (typeof documents)["\n      query PoolStatistics {\n        statistics {\n          diskUsage {\n            used\n            usedLastMonth\n            free\n            total\n          }\n          poolUsage {\n            nbChunk\n            nbChunkLastMonth\n            nbChunkRange {\n              time\n              value\n            }\n\n            nbRef\n            nbRefLastMonth\n\n            size\n            compressedSize\n            compressedSizeLastMonth\n            compressedSizeRange {\n              time\n              value\n            }\n\n            unusedSize\n          }\n        }\n      }\n    "];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment JobBackupData on JobBackupData {\n    host\n    number\n    ip\n    startDate\n  }\n"): (typeof documents)["\n  fragment JobBackupData on JobBackupData {\n    host\n    number\n    ip\n    startDate\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment JobRestoreData on JobRestoreData {\n    host\n    number\n    ip\n    startDate\n    destinationDirectory\n    files {\n      share\n      selection\n    }\n  }\n"): (typeof documents)["\n  fragment JobRestoreData on JobRestoreData {\n    host\n    number\n    ip\n    startDate\n    destinationDirectory\n    files {\n      share\n      selection\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment JobRemoveData on JobRemoveData {\n    host\n    number\n    startDate\n  }\n"): (typeof documents)["\n  fragment JobRemoveData on JobRemoveData {\n    host\n    number\n    startDate\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment JobCleanupData on JobCleanupData {\n    target\n  }\n"): (typeof documents)["\n  fragment JobCleanupData on JobCleanupData {\n    target\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment JobFsckData on JobFsckData {\n    dryRun\n    verifyChunks\n  }\n"): (typeof documents)["\n  fragment JobFsckData on JobFsckData {\n    dryRun\n    verifyChunks\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment BackupTaskState on JobBackupTaskState {\n    backupExecutionState: executionState\n    backupErrorState: errorState\n    backupErrorMessage: errorMessage\n    globalProgression: progression {\n      startDate\n      startTransferDate\n      endTransferDate\n      fileSize\n      newFileSize\n      modifiedFileSize\n      compressedFileSize\n      newCompressedFileSize\n      modifiedCompressedFileSize\n      fileCount\n      newFileCount\n      modifiedFileCount\n      removedFileCount\n      errorCount\n      speed\n      percent\n      progressCurrent\n      progressMax\n    }\n    preCommandStates {\n      executionState\n      command {\n        command\n      }\n    }\n    shareStates {\n      share\n      executionState\n      backupProgression {\n        startDate\n        startTransferDate\n        endTransferDate\n        fileSize\n        newFileSize\n        modifiedFileSize\n        compressedFileSize\n        newCompressedFileSize\n        modifiedCompressedFileSize\n        fileCount\n        newFileCount\n        modifiedFileCount\n        removedFileCount\n        errorCount\n        speed\n        percent\n        progressCurrent\n        progressMax\n      }\n      fileListProgression {\n        fileSize\n        newFileSize\n        modifiedFileSize\n        newFileCount\n        modifiedFileCount\n        removedFileCount\n      }\n    }\n    postCommandStates {\n      executionState\n      command {\n        command\n      }\n    }\n  }\n"): (typeof documents)["\n  fragment BackupTaskState on JobBackupTaskState {\n    backupExecutionState: executionState\n    backupErrorState: errorState\n    backupErrorMessage: errorMessage\n    globalProgression: progression {\n      startDate\n      startTransferDate\n      endTransferDate\n      fileSize\n      newFileSize\n      modifiedFileSize\n      compressedFileSize\n      newCompressedFileSize\n      modifiedCompressedFileSize\n      fileCount\n      newFileCount\n      modifiedFileCount\n      removedFileCount\n      errorCount\n      speed\n      percent\n      progressCurrent\n      progressMax\n    }\n    preCommandStates {\n      executionState\n      command {\n        command\n      }\n    }\n    shareStates {\n      share\n      executionState\n      backupProgression {\n        startDate\n        startTransferDate\n        endTransferDate\n        fileSize\n        newFileSize\n        modifiedFileSize\n        compressedFileSize\n        newCompressedFileSize\n        modifiedCompressedFileSize\n        fileCount\n        newFileCount\n        modifiedFileCount\n        removedFileCount\n        errorCount\n        speed\n        percent\n        progressCurrent\n        progressMax\n      }\n      fileListProgression {\n        fileSize\n        newFileSize\n        modifiedFileSize\n        newFileCount\n        modifiedFileCount\n        removedFileCount\n      }\n    }\n    postCommandStates {\n      executionState\n      command {\n        command\n      }\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment RestoreTaskState on JobRestoreTaskState {\n    restoreExecutionState: executionState\n    restoreErrorState: errorState\n    restoreErrorMessage: errorMessage\n    restoreProgression: globalProgression {\n      startDate\n      startTransferDate\n      endTransferDate\n      fileSize\n      newFileSize\n      modifiedFileSize\n      compressedFileSize\n      newCompressedFileSize\n      modifiedCompressedFileSize\n      fileCount\n      newFileCount\n      modifiedFileCount\n      removedFileCount\n      errorCount\n      speed\n      percent\n      progressCurrent\n      progressMax\n    }\n  }\n"): (typeof documents)["\n  fragment RestoreTaskState on JobRestoreTaskState {\n    restoreExecutionState: executionState\n    restoreErrorState: errorState\n    restoreErrorMessage: errorMessage\n    restoreProgression: globalProgression {\n      startDate\n      startTransferDate\n      endTransferDate\n      fileSize\n      newFileSize\n      modifiedFileSize\n      compressedFileSize\n      newCompressedFileSize\n      modifiedCompressedFileSize\n      fileCount\n      newFileCount\n      modifiedFileCount\n      removedFileCount\n      errorCount\n      speed\n      percent\n      progressCurrent\n      progressMax\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment RemoveTaskState on JobRemoveState {\n    removeExecutionState: executionState\n    removeErrorState: errorState\n    removeErrorMessage: errorMessage\n  }\n"): (typeof documents)["\n  fragment RemoveTaskState on JobRemoveState {\n    removeExecutionState: executionState\n    removeErrorState: errorState\n    removeErrorMessage: errorMessage\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment CleanerTaskState on JobCleanerTaskState {\n    cleanerExecutionState: executionState\n    cleanerErrorState: errorState\n    cleanerErrorMessage: errorMessage\n    cleanerProgress: progression {\n      progressMax\n      progressCurrent\n      fileSize\n      compressedFileSize\n    }\n  }\n"): (typeof documents)["\n  fragment CleanerTaskState on JobCleanerTaskState {\n    cleanerExecutionState: executionState\n    cleanerErrorState: errorState\n    cleanerErrorMessage: errorMessage\n    cleanerProgress: progression {\n      progressMax\n      progressCurrent\n      fileSize\n      compressedFileSize\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment FsckTaskState on JobFsckTaskState {\n    fsckExecutionState: executionState\n    fsckErrorState: errorState\n    fsckErrorMessage: errorMessage\n    dryRun\n    refcntProgression {\n      progressMax\n      progressCurrent\n      errorCount\n      totalCount\n    }\n    unusedProgression {\n      progressMax\n      progressCurrent\n      inNothing\n      inRefcnt\n      inUnused\n      missing\n    }\n    chunkProgression {\n      progressMax\n      progressCurrent\n      errorCount\n      totalCount\n    }\n  }\n"): (typeof documents)["\n  fragment FsckTaskState on JobFsckTaskState {\n    fsckExecutionState: executionState\n    fsckErrorState: errorState\n    fsckErrorMessage: errorMessage\n    dryRun\n    refcntProgression {\n      progressMax\n      progressCurrent\n      errorCount\n      totalCount\n    }\n    unusedProgression {\n      progressMax\n      progressCurrent\n      inNothing\n      inRefcnt\n      inUnused\n      missing\n    }\n    chunkProgression {\n      progressMax\n      progressCurrent\n      errorCount\n      totalCount\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment Job on Job {\n    jobId\n    kind\n    status\n    timestamp\n    host\n    failedReason\n    data {\n      ...JobBackupData\n      ...JobRestoreData\n      ...JobRemoveData\n      ...JobCleanupData\n      ...JobFsckData\n    }\n    progress {\n      ...BackupTaskState\n      ...RestoreTaskState\n      ...RemoveTaskState\n      ...CleanerTaskState\n      ...FsckTaskState\n    }\n  }\n"): (typeof documents)["\n  fragment Job on Job {\n    jobId\n    kind\n    status\n    timestamp\n    host\n    failedReason\n    data {\n      ...JobBackupData\n      ...JobRestoreData\n      ...JobRemoveData\n      ...JobCleanupData\n      ...JobFsckData\n    }\n    progress {\n      ...BackupTaskState\n      ...RestoreTaskState\n      ...RemoveTaskState\n      ...CleanerTaskState\n      ...FsckTaskState\n    }\n  }\n"];
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