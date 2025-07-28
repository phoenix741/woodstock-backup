import type {
  BackupStatusDto,
  HostAvailibilityState
} from '@/generated/graphql';
import { format, formatDuration, intervalToDuration } from 'date-fns';
import numeral from 'numeral';
import { computed } from 'vue';
import vuetify from '../../plugins/vuetify';
import {
  getBackupStatusLabel,
  getBackupStatusColor,
  getBackupStatusIcon,
  getBackupStatusKey
} from '@/utils/backup-status';

export const BackupStatusDisabled = 'Disabled';
export const BackupStatusIdle = 'Idle';
export type DeviceBackupStatus = BackupStatusDto | typeof BackupStatusDisabled | typeof BackupStatusIdle;

// Get Vuetify themes (light and dark)
const vuetifyThemes = vuetify.theme.themes.value;
const lightColors = vuetifyThemes.light.colors;
const darkColors = vuetifyThemes.dark.colors;

const currentTheme = computed(() => {
  return vuetify.theme.global.current.value.dark ? darkColors : lightColors;
});

/**
 * Get the backup state from a host object
 *
 * @param host - Host object from GraphQL query (can be partial/fragment)
 */
export function getState(host: {
  configuration?: { schedule?: { activated?: boolean | null } | null } | null;
  lastBackup?: { status?: unknown } | null;
}): DeviceBackupStatus {
  if (!host.configuration?.schedule?.activated) {
    return BackupStatusDisabled;
  } else if (host.lastBackup?.status) {
    return host.lastBackup.status as DeviceBackupStatus;
  }
  return BackupStatusIdle;
}

/**
 * Get a unique string key for a device state (useful for dictionaries/maps)
 */
export function getStateKey(state: DeviceBackupStatus): string {
  if (state === BackupStatusDisabled) return 'Disabled';
  if (state === BackupStatusIdle) return 'Idle';
  return getBackupStatusKey(state);
}

/**
 * Get human-readable text for a device state (for display in chips)
 * Handles both special statuses (Disabled, Idle) and BackupStatusDto
 */
export function getStateText(state: DeviceBackupStatus): string {
  if (state === BackupStatusDisabled) {
    return 'Disabled';
  }
  if (state === BackupStatusIdle) {
    return 'Idle';
  }
  return getBackupStatusLabel(state);
}

/**
 * Get color for a device state
 */
export function getStateColor(state: DeviceBackupStatus): string {
  // Handle special states (Disabled, Idle)
  if (state === BackupStatusDisabled) {
    return currentTheme.value.secondary;
  }
  if (state === BackupStatusIdle) {
    return currentTheme.value.primary;
  }

  return getBackupStatusColor(state);
}

/**
 * Get Material Design Icon for a state
 */
export function getStatusIcon(status: unknown): string {
  return getBackupStatusIcon(status);
}

/**
 * Get color for host availability state
 */
export function getAvailabilityColor(availibilityState: HostAvailibilityState | undefined | null): string {
  switch (availibilityState) {
    case 'ONLINE':
      return currentTheme.value.success;
    case 'OFFLINE':
      return currentTheme.value.error;
    case 'UNKNOWN':
      return currentTheme.value.primary;
    default:
      return currentTheme.value.primary;
  }
}

export function toDuration(age: number) {
  const duration = intervalToDuration({ start: 0, end: age });
  if (duration.seconds) {
    duration.minutes = (duration.minutes ?? 0) + 1;
    duration.seconds = 0;
  }

  if (duration.years) {
    duration.months = (duration.months ?? 0) + 1;
    duration.days = 0;
    duration.hours = 0;
    duration.minutes = 0;
    duration.seconds = 0;
  } else if (duration.months) {
    duration.days = (duration.days ?? 0) + 1;
    duration.hours = 0;
    duration.minutes = 0;
    duration.seconds = 0;
  } else if (duration.days) {
    duration.hours = (duration.hours ?? 0) + 1;
    duration.minutes = 0;
    duration.seconds = 0;
  } else if (duration.hours) {
    duration.minutes = (duration.minutes ?? 0) + 1;
  }

  return formatDuration(duration);
}

export function toDateTime(value: string | number | Date) {
  return format(value, 'MM/dd/yyyy HH:mm');
}

export function toDate(value: string | number | Date) {
  return format(value, 'MM/dd/yyyy');
}

export function toPercent(value?: number) {
  if (value === null || value === undefined) return '';
  return numeral(value / 100).format('0.00%');
}

export function toNumber(value?: number) {
  if (value === null || value === undefined) return '';
  return numeral(value).format('0,000');
}
