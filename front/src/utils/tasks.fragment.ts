import { graphql } from '@/generated';

export const JobBackupDataFragmentDoc = graphql(/* GraphQL */ `
  fragment JobBackupData on JobBackupData {
    host
    number
    ip
    startDate
  }
`);

export const JobRestoreDataFragmentDoc = graphql(/* GraphQL */ `
  fragment JobRestoreData on JobRestoreData {
    host
    number
    ip
    startDate
    destinationDirectory
    files {
      share
      selection
    }
  }
`);

export const JobRemoveDataFragmentDoc = graphql(/* GraphQL */ `
  fragment JobRemoveData on JobRemoveData {
    host
    number
    startDate
  }
`);

export const JobCleanupDataFragmentDoc = graphql(/* GraphQL */ `
  fragment JobCleanupData on JobCleanupData {
    target
  }
`);

export const JobFsckDataFragmentDoc = graphql(/* GraphQL */ `
  fragment JobFsckData on JobFsckData {
    dryRun
    verifyChunks
  }
`);

export const BackupTaskStateFragmentDoc = graphql(/* GraphQL */ `
  fragment BackupTaskState on JobBackupTaskState {
    backupExecutionState: executionState
    backupErrorState: errorState
    backupErrorMessage: errorMessage
    globalProgression: progression {
      startDate
      startTransferDate
      endTransferDate
      fileSize
      newFileSize
      modifiedFileSize
      compressedFileSize
      newCompressedFileSize
      modifiedCompressedFileSize
      fileCount
      newFileCount
      modifiedFileCount
      removedFileCount
      errorCount
      speed
      percent
      progressCurrent
      progressMax
    }
    preCommandStates {
      executionState
      command {
        command
      }
    }
    shareStates {
      share
      executionState
      backupProgression {
        startDate
        startTransferDate
        endTransferDate
        fileSize
        newFileSize
        modifiedFileSize
        compressedFileSize
        newCompressedFileSize
        modifiedCompressedFileSize
        fileCount
        newFileCount
        modifiedFileCount
        removedFileCount
        errorCount
        speed
        percent
        progressCurrent
        progressMax
      }
      fileListProgression {
        fileSize
        newFileSize
        modifiedFileSize
        newFileCount
        modifiedFileCount
        removedFileCount
      }
    }
    postCommandStates {
      executionState
      command {
        command
      }
    }
  }
`);

export const RestoreTaskStateFragmentDoc = graphql(/* GraphQL */ `
  fragment RestoreTaskState on JobRestoreTaskState {
    restoreExecutionState: executionState
    restoreErrorState: errorState
    restoreErrorMessage: errorMessage
    restoreProgression: globalProgression {
      startDate
      startTransferDate
      endTransferDate
      fileSize
      newFileSize
      modifiedFileSize
      compressedFileSize
      newCompressedFileSize
      modifiedCompressedFileSize
      fileCount
      newFileCount
      modifiedFileCount
      removedFileCount
      errorCount
      speed
      percent
      progressCurrent
      progressMax
    }
  }
`);

export const RemoveTaskStateFragmentDoc = graphql(/* GraphQL */ `
  fragment RemoveTaskState on JobRemoveState {
    removeExecutionState: executionState
    removeErrorState: errorState
    removeErrorMessage: errorMessage
  }
`);

export const CleanerTaskStateFragmentDoc = graphql(/* GraphQL */ `
  fragment CleanerTaskState on JobCleanerTaskState {
    cleanerExecutionState: executionState
    cleanerErrorState: errorState
    cleanerErrorMessage: errorMessage
    cleanerProgress: progression {
      progressMax
      progressCurrent
      fileSize
      compressedFileSize
    }
  }
`);

export const FsckTaskStateFragmentDoc = graphql(/* GraphQL */ `
  fragment FsckTaskState on JobFsckTaskState {
    fsckExecutionState: executionState
    fsckErrorState: errorState
    fsckErrorMessage: errorMessage
    dryRun
    refcntProgression {
      progressMax
      progressCurrent
      errorCount
      totalCount
    }
    unusedProgression {
      progressMax
      progressCurrent
      inNothing
      inRefcnt
      inUnused
      missing
    }
    chunkProgression {
      progressMax
      progressCurrent
      errorCount
      totalCount
    }
  }
`);

export const JobFragmentDoc = graphql(/* GraphQL */ `
  fragment Job on Job {
    jobId
    kind
    status
    timestamp
    host
    failedReason
    data {
      ...JobBackupData
      ...JobRestoreData
      ...JobRemoveData
      ...JobCleanupData
      ...JobFsckData
    }
    progress {
      ...BackupTaskState
      ...RestoreTaskState
      ...RemoveTaskState
      ...CleanerTaskState
      ...FsckTaskState
    }
  }
`);
