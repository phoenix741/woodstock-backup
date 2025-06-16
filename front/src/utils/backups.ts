import { useFragment } from '@/generated';
import {
  BackupDocument,
  BackupsBrowseDocument,
  BackupsDocument,
  BackupStatus,
  FragmentFileDescriptionFragmentDoc,
  SharesBrowseDocument,
} from '@/generated/graphql';
import { ApolloClient } from '@apollo/client/core';
import { useQuery } from '@vue/apollo-composable';
import { computed } from 'vue';
import vuetify from '../plugins/vuetify';

// On récupère les thèmes de Vuetify (light et dark)
const vuetifyThemes = vuetify.theme.themes.value;
const lightColors = vuetifyThemes.light.colors;
const darkColors = vuetifyThemes.dark.colors;

const currentTheme = computed(() => {
  return vuetify.theme.global.current.value.dark ? darkColors : lightColors;
});

export function getBackupStatusColor(backupStatus: BackupStatus | undefined | null) {
  switch (backupStatus) {
    case BackupStatus.Completed:
      return currentTheme.value.success;
    case BackupStatus.Aborted:
    case BackupStatus.Failed:
      return currentTheme.value.error;
    case BackupStatus.Finishing:
    case BackupStatus.InProgress:
      return currentTheme.value.info;
    default:
      return currentTheme.value.primary;
  }
}

export function useShare(deviceId: string, backupNumber: number) {
  const { result: data, loading: isFetching } = useQuery(SharesBrowseDocument, {
    hostname: deviceId,
    number: backupNumber,
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
  const { result: data, loading: isFetching } = useQuery(BackupsDocument, {
    hostname: deviceId,
  });

  const backups = computed(() => data.value?.backups);

  return {
    backups,
    isFetching,
  };
}

export function useBackup(deviceId: string, backupNumber: number) {
  const { result: data, loading: isFetching } = useQuery(BackupDocument, {
    hostname: deviceId,
    number: backupNumber,
  });

  const backup = computed(() => data.value?.backup);

  return {
    backup,
    isFetching,
  };
}

export function useBackupsBrowse(deviceId: string, backupNumber: number) {
  const { shares, isFetching } = useShare(deviceId, backupNumber);

  const browse = async <T>(client: ApolloClient<T>, sharePath: string, path: string) => {
    const { data } = await client.query({
      query: BackupsBrowseDocument,
      variables: {
        hostname: deviceId,
        number: backupNumber,
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
