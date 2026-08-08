import { ArchiveFormat, ArchiveProfilesDocument, HostSelectionMode } from '@/generated/graphql';
import { useQuery } from '@vue/apollo-composable';
import { computed } from 'vue';

/**
 * Composable for fetching the list of archive profiles (`archiving.yml`, read-only).
 */
export function useArchiveProfiles() {
  const { result, loading: isArchiveProfilesFetching } = useQuery(ArchiveProfilesDocument, null, {
    fetchPolicy: 'cache-and-network',
  });

  const profiles = computed(() => result.value?.archiveProfiles ?? []);

  return {
    profiles,
    isArchiveProfilesFetching,
  };
}

/**
 * Composable for fetching a single archive profile by name.
 *
 * There is no dedicated single-profile query — the list is small (hand-authored
 * YAML), so we fetch it once and find the profile client-side.
 */
export function useArchiveProfile(profileName: string) {
  const { profiles, isArchiveProfilesFetching } = useArchiveProfiles();

  const profile = computed(() => profiles.value.find((p) => p.name === profileName));

  return {
    profile,
    isArchiveProfilesFetching,
  };
}

export function archiveFormatLabel(format: ArchiveFormat): string {
  switch (format) {
    case ArchiveFormat.Tar:
      return 'tar';
    case ArchiveFormat.TarGz:
      return 'tar.gz';
    case ArchiveFormat.TarXz:
      return 'tar.xz';
    case ArchiveFormat.TarZstd:
      return 'tar.zst';
    case ArchiveFormat.Dir:
      return 'dir (incremental mirror)';
    default:
      return format;
  }
}

/** Whether `format` is compressed and so honors `compressionLevel` (`tar`/`dir` don't compress). */
export function isCompressedFormat(format: ArchiveFormat): boolean {
  return format === ArchiveFormat.TarGz || format === ArchiveFormat.TarXz || format === ArchiveFormat.TarZstd;
}

export function hostSelectionModeLabel(mode: HostSelectionMode): string {
  switch (mode) {
    case HostSelectionMode.All:
      return 'All hosts';
    case HostSelectionMode.Glob:
      return 'Filter (glob pattern)';
    case HostSelectionMode.Include:
      return 'Include list';
    case HostSelectionMode.Exclude:
      return 'Exclude list';
    default:
      return mode;
  }
}

/** One-line summary of a profile's host selection, for the list table. */
export function hostSelectionSummary(profile: {
  hostSelectionMode: HostSelectionMode;
  hostSelectionPattern?: string | null;
  hostSelectionHosts?: string[] | null;
}): string {
  switch (profile.hostSelectionMode) {
    case HostSelectionMode.All:
      return 'All hosts';
    case HostSelectionMode.Glob:
      return `Filter: ${profile.hostSelectionPattern}`;
    case HostSelectionMode.Include:
      return `Include: ${(profile.hostSelectionHosts ?? []).join(', ')}`;
    case HostSelectionMode.Exclude:
      return `Exclude: ${(profile.hostSelectionHosts ?? []).join(', ')}`;
    default:
      return '';
  }
}
