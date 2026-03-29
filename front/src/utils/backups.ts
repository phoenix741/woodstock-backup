import { useFragment } from '@/generated';
import {
  BackupDocument,
  BackupsBrowseDocument,
  BackupsDocument,
  FragmentFileDescriptionFragmentDoc,
  JobStatus,
  SharesBrowseDocument,
} from '@/generated/graphql';
import { ApolloClient } from '@apollo/client/core';
import { useQuery, useSubscription } from '@vue/apollo-composable';
import gql from 'graphql-tag';
import { computed, watch } from 'vue';

/**
 * Subscription document for real-time backup status updates.
 * Will be fully typed after regenerating the codegen (once the server is deployed).
 */
const BACKUP_UPDATED_SUBSCRIPTION = gql`
  subscription BackupUpdated($hostname: String!) {
    backupUpdated(hostname: $hostname) {
      id
      number
      status {
        statusType
        finishingStage
        abortingStage
        failedStage
        removingStage
      }
      startDate
      endDate
      errorCount
      fileCount
      newFileCount
      existingFileCount
      removedFileCount
      modifiedFileCount
      fileSize
      newFileSize
      existingFileSize
      speed
    }
  }
`;

/**
 * Minimal subscription to detect the completion of a remove job.
 * Uses the host/kind filters added to jobUpdated.
 */
const JOB_REMOVE_SUBSCRIPTION = gql`
  subscription JobRemoveUpdated($host: String, $kind: String) {
    jobUpdated(host: $host, kind: $kind) {
      jobId
      status
    }
  }
`;

/**
 * Subscription to detect the start or completion of a backup job.
 * Triggers a list refetch so the cache is invalidated even if
 * the user navigated away from the Tasks page.
 */
const JOB_BACKUP_SUBSCRIPTION = gql`
  subscription JobBackupUpdated($host: String, $kind: String) {
    jobUpdated(host: $host, kind: $kind) {
      jobId
      status
    }
  }
`;

// Re-export backup status utilities for convenience
export {
  getBackupStatusColor,
  getBackupStatusLabel as getBackupStatusText,
  getBackupStatusIcon,
  getBackupStatusKey,
} from './backup-status';

export function useShare(deviceId: string, backupId: string) {
  const { result: data, loading: isFetching } = useQuery(SharesBrowseDocument, {
    hostname: deviceId,
    id: backupId,
  });

  const shares = computed(() =>
    data.value?.backup.shares
      .map((fragment) => useFragment(FragmentFileDescriptionFragmentDoc, fragment))
      .sort((a, b) => a.type.localeCompare(b.type) || a.path.localeCompare(b.path)),
  );

  return {
    shares,
    isFetching,
  };
}

export function useBackups(deviceId: string) {
  const {
    result: data,
    loading: isFetching,
    subscribeToMore,
    refetch,
  } = useQuery(BackupsDocument, {
    hostname: deviceId,
  });

  // Real-time updates: refreshes the status of existing backups
  // (running, completed, being removed).
  // Does NOT insert new rows: insertion is handled by an explicit refetch.
  subscribeToMore(() => ({
    document: BACKUP_UPDATED_SUBSCRIPTION,
    variables: { hostname: deviceId },
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    updateQuery: (previousResult, { subscriptionData }: any) => {
      const updatedBackup = subscriptionData.data?.backupUpdated;
      if (!updatedBackup || !previousResult?.backups) return previousResult;

      const list = [...previousResult.backups];
      const idx = list.findIndex((b) => b.id === updatedBackup.id);
      if (idx >= 0) {
        // Only update existing entries (status change)
        list[idx] = updatedBackup;
      }
      // Do NOT insert new entries here:
      // a new backup will appear after the next automatic refetch.
      return { ...previousResult, backups: list };
    },
  }));

  // Detect the end of a removal to drop the entry from the list.
  // backupUpdated returns nothing when the backup is already deleted from disk,
  // so we watch jobUpdated(kind=remove) and reload the list on completion.
  const { result: removeResult } = useSubscription(JOB_REMOVE_SUBSCRIPTION, {
    host: deviceId,
    kind: 'remove',
  });

  watch(removeResult, (val) => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    if ((val as any)?.jobUpdated?.status === JobStatus.Completed) {
      // Reload the full list: the deleted backup will no longer be in it.
      refetch();
    }
  });

  // Invalidate the cache when a backup job starts or finishes.
  // Use case: the user triggers a backup (navigates to Tasks), then comes back
  // to the Backups page. Without this, the new in-progress (or completed) backup
  // would not appear because backupUpdated does not insert new entries and the
  // Apollo cache was not refreshed while the user was away.
  const { result: backupJobResult } = useSubscription(JOB_BACKUP_SUBSCRIPTION, {
    host: deviceId,
    kind: 'backup',
  });

  watch(backupJobResult, (val) => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const status = (val as any)?.jobUpdated?.status;
    if (status === JobStatus.Started || status === JobStatus.Completed || status === JobStatus.Failed) {
      refetch();
    }
  });

  const backups = computed(() => data.value?.backups);

  return {
    backups,
    isFetching,
  };
}

export function useBackup(deviceId: string, backupId: string) {
  const {
    result: data,
    loading: isFetching,
    subscribeToMore,
  } = useQuery(BackupDocument, {
    hostname: deviceId,
    id: backupId,
  });

  // Real-time update: replaces the current backup data on every state change.
  subscribeToMore(() => ({
    document: BACKUP_UPDATED_SUBSCRIPTION,
    variables: { hostname: deviceId },
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    updateQuery: (previousResult, { subscriptionData }: any) => {
      const updatedBackup = subscriptionData.data?.backupUpdated;
      if (!updatedBackup || updatedBackup.id !== backupId) return previousResult;
      return { ...previousResult, backup: updatedBackup };
    },
  }));

  const backup = computed(() => data.value?.backup);

  return {
    backup,
    isFetching,
  };
}

export function useBackupsBrowse(deviceId: string, backupId: string) {
  const { shares, isFetching } = useShare(deviceId, backupId);

  const browse = async <T>(client: ApolloClient<T>, sharePath: string, path: string) => {
    const { data } = await client.query({
      query: BackupsBrowseDocument,
      variables: {
        hostname: deviceId,
        id: backupId,
        sharePath,
        path,
      },
    });

    return data.backup.files
      .map((fragment) => useFragment(FragmentFileDescriptionFragmentDoc, fragment))
      .sort((a, b) => a.type.localeCompare(b.type) || a.path.localeCompare(b.path));
  };

  return {
    shares,
    isFetching,

    browse,
  };
}
