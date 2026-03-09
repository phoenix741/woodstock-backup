import type {
  BackupStatusDto,
  HostAvailibilityState
} from '@/generated/graphql';
import { computed } from 'vue';
import vuetify from '../../plugins/vuetify';
import {
  getBackupStatusLabel,
  getBackupStatusColor,
  getBackupStatusIcon,
  getBackupStatusKey
} from '@/utils/backup-status';
import {
  formatDateTimeValue,
  formatDateValue,
  formatDurationValue,
  formatNumberValue,
  formatPercentValue,
  parseDateTime,
} from '@/utils/formatting';

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
  return formatDurationValue(age);
}

export function toDateTime(value: string | number | Date) {
  return formatDateTimeValue(value);
}

export function toDate(value: string | number | Date) {
  return formatDateValue(value);
}

export function toPercent(value?: number) {
  return formatPercentValue(value);
}

export function toNumber(value?: number | bigint) {
  return formatNumberValue(value);
}

export { parseDateTime };
