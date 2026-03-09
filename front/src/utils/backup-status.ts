/**
 * Backup Status Utilities
 *
 * This module provides utilities for handling backup status types, colors, labels and icons.
 * It supports both regular BackupStatusDto and GraphQL fragments.
 */

import type {
  BackupStatusDto,
  BackupStatusFieldsFragment,
} from '@/generated/graphql';
import {
  BackupStatusTypeDto,
  FinishingStageDto,
  AbortingStageDto,
  FailedStageDto,
  RemovingStageDto,
} from '@/generated/graphql';
import { computed } from 'vue';
import vuetify from '../plugins/vuetify';

/**
 * Union type that accepts both full BackupStatusDto and GraphQL fragments
 */
export type BackupStatusLike =
  | BackupStatusDto
  | BackupStatusFieldsFragment
  | {
      __typename?: 'BackupStatusDto';
      statusType: BackupStatusTypeDto;
      finishingStage?: FinishingStageDto | null;
      abortingStage?: AbortingStageDto | null;
      failedStage?: FailedStageDto | null;
      removingStage?: RemovingStageDto | null;
    };

// Get Vuetify themes (light and dark)
const vuetifyThemes = vuetify.theme.themes.value;
const lightColors = vuetifyThemes.light.colors;
const darkColors = vuetifyThemes.dark.colors;

const currentTheme = computed(() => {
  return vuetify.theme.global.current.value.dark ? darkColors : lightColors;
});

function asThemeColor(value: unknown): string {
  return typeof value === 'string' ? value : String(value ?? '');
}

/**
 * Type guard to check if a value is a valid backup status
 */
function isBackupStatus(status: unknown): status is BackupStatusLike {
  return (
    status !== null &&
    status !== undefined &&
    typeof status === 'object' &&
    'statusType' in status &&
    typeof (status as BackupStatusLike).statusType === 'string'
  );
}

/**
 * Stage label translations
 */
const STAGE_LABELS = {
  finishing: {
    [FinishingStageDto.ToCompact]: 'Compacting',
    [FinishingStageDto.ToCountRef]: 'Counting References',
    [FinishingStageDto.ToAddInPool]: 'Adding to Pool',
  },
  aborting: {
    [AbortingStageDto.ToCompact]: 'Compacting',
    [AbortingStageDto.ToCountRef]: 'Counting References',
    [AbortingStageDto.ToAddInPool]: 'Adding to Pool',
  },
  failed: {
    [FailedStageDto.Compact]: 'Compacting',
    [FailedStageDto.RefCount]: 'Counting References',
    [FailedStageDto.InPool]: 'Adding to Pool',
  },
  removing: {
    [RemovingStageDto.ToRemove]: 'Preparation',
    [RemovingStageDto.ToRemoveInPool]: 'Removing from Pool',
    [RemovingStageDto.RemoveFromHost]: 'Removing from Host',
  },
} as const;

/**
 * Stage colors for Finishing status
 */
const FINISHING_COLORS = {
  [FinishingStageDto.ToCompact]: '#FF9800',    // Orange
  [FinishingStageDto.ToCountRef]: '#FFC107',   // Amber/Yellow
  [FinishingStageDto.ToAddInPool]: '#03A9F4',  // Light Blue
} as const;

/**
 * Stage colors for Removing status
 */
const REMOVING_COLORS = {
  [RemovingStageDto.ToRemove]: '#FFCDD2',        // Very Light Red
  [RemovingStageDto.ToRemoveInPool]: '#EF9A9A',  // Light Red
  [RemovingStageDto.RemoveFromHost]: '#E57373',  // Medium Red
} as const;

/**
 * Get human-readable label for a backup status
 */
export function getBackupStatusLabel(status: unknown): string {
  if (!isBackupStatus(status)) {
    return 'Unknown';
  }

  switch (status.statusType) {
    case BackupStatusTypeDto.InProgress:
      return 'In Progress';

    case BackupStatusTypeDto.Finishing: {
      const stage = status.finishingStage;
      const stageLabel = stage ? STAGE_LABELS.finishing[stage as FinishingStageDto] : null;
      return stageLabel ? `Finishing - ${stageLabel}` : 'Finishing';
    }

    case BackupStatusTypeDto.Aborting: {
      const stage = status.abortingStage;
      const stageLabel = stage ? STAGE_LABELS.aborting[stage as AbortingStageDto] : null;
      return stageLabel ? `Aborting - ${stageLabel}` : 'Aborting';
    }

    case BackupStatusTypeDto.Completed:
      return 'Completed';

    case BackupStatusTypeDto.Aborted:
      return 'Aborted';

    case BackupStatusTypeDto.Failed: {
      const stage = status.failedStage;
      const stageLabel = stage ? STAGE_LABELS.failed[stage as FailedStageDto] : null;
      return stageLabel ? `Failed - ${stageLabel}` : 'Failed';
    }

    case BackupStatusTypeDto.Removing: {
      const stage = status.removingStage;
      const stageLabel = stage ? STAGE_LABELS.removing[stage as RemovingStageDto] : null;
      return stageLabel ? `Removing - ${stageLabel}` : 'Removing';
    }

    default:
      return 'Unknown';
  }
}

/**
 * Get color for a backup status
 */
export function getBackupStatusColor(status: unknown): string {
  if (!isBackupStatus(status)) {
    return asThemeColor(currentTheme.value.primary);
  }

  switch (status.statusType) {
    case BackupStatusTypeDto.Completed:
      return asThemeColor(currentTheme.value.success);

    case BackupStatusTypeDto.Aborted:
    case BackupStatusTypeDto.Failed:
      return asThemeColor(currentTheme.value.error);

    case BackupStatusTypeDto.InProgress:
      return asThemeColor(currentTheme.value.info);

    case BackupStatusTypeDto.Finishing: {
      const stage = status.finishingStage as FinishingStageDto;
      return FINISHING_COLORS[stage] || asThemeColor(currentTheme.value.info);
    }

    case BackupStatusTypeDto.Aborting:
      return '#FF6F00'; // Dark Orange

    case BackupStatusTypeDto.Removing: {
      const stage = status.removingStage as RemovingStageDto;
      return REMOVING_COLORS[stage] || '#EF5350'; // Red
    }

    default:
      return asThemeColor(currentTheme.value.primary);
  }
}

/**
 * Get Material Design Icon for a backup status
 */
export function getBackupStatusIcon(status: unknown): string {
  if (!isBackupStatus(status)) {
    return 'mdi-help-circle';
  }

  switch (status.statusType) {
    case BackupStatusTypeDto.InProgress:
      return 'mdi-progress-upload';

    case BackupStatusTypeDto.Finishing:
      switch (status.finishingStage) {
        case FinishingStageDto.ToCompact:
          return 'mdi-package-variant';
        case FinishingStageDto.ToCountRef:
          return 'mdi-counter';
        case FinishingStageDto.ToAddInPool:
          return 'mdi-database-arrow-down';
        default:
          return 'mdi-cog';
      }

    case BackupStatusTypeDto.Aborting:
      return 'mdi-stop-circle-outline';

    case BackupStatusTypeDto.Completed:
      return 'mdi-check-circle';

    case BackupStatusTypeDto.Aborted:
      return 'mdi-close-circle';

    case BackupStatusTypeDto.Failed:
      return 'mdi-alert-circle';

    case BackupStatusTypeDto.Removing:
      return 'mdi-delete';

    default:
      return 'mdi-help-circle';
  }
}

/**
 * Get a unique string key for a backup status (useful for dictionaries/maps)
 */
export function getBackupStatusKey(status: unknown): string {
  if (!isBackupStatus(status)) {
    return 'unknown';
  }

  let key: string = status.statusType;

  if (status.finishingStage) key += `_${status.finishingStage}`;
  if (status.abortingStage) key += `_${status.abortingStage}`;
  if (status.failedStage) key += `_${status.failedStage}`;
  if (status.removingStage) key += `_${status.removingStage}`;

  return key;
}
