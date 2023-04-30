import { useFragment } from '@/generated';
import {
  BackupsBrowseDocument,
  BackupsDocument,
  FragmentFileDescriptionFragmentDoc,
  SharesBrowseDocument,
} from '@/generated/graphql';
import { useApolloClient, useQuery } from '@vue/apollo-composable';
import { computed } from 'vue';

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

export function useBackupsBrowse(deviceId: string, backupNumber: number) {
  const { client } = useApolloClient();

  const { result: data, loading: isFetching } = useQuery(SharesBrowseDocument, {
    hostname: deviceId,
    number: backupNumber,
  });

  const browse = async (sharePath: string, path: string) => {
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

  const shares = computed(() =>
    data.value?.backup.shares
      .map((fragment) => useFragment(FragmentFileDescriptionFragmentDoc, fragment))
      .sort((a, b) => a.type.localeCompare(b.type) || a.path.localeCompare(b.path)),
  );

  return {
    shares,
    isFetching,

    browse,
  };
}
