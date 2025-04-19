import { HostAvailibilityState } from '@/generated/graphql';
import { format, formatDuration, intervalToDuration } from 'date-fns';
import numeral from 'numeral';
import { computed } from 'vue';
import vuetify from '../../plugins/vuetify';

// On récupère les thèmes de Vuetify (light et dark)
const vuetifyThemes = vuetify.theme.themes.value;
const lightColors = vuetifyThemes.light.colors;
const darkColors = vuetifyThemes.dark.colors;

const currentTheme = computed(() => {
  return vuetify.theme.global.current.value.dark ? darkColors : lightColors;
});

export function getState(host: {
  configuration?: { schedule?: { activated?: boolean | null } | null } | null;
  lastBackupState?: string | null;
  lastBackup?: { completed?: boolean | null } | null;
}) {
  if (!host.configuration?.schedule?.activated) {
    return 'disabled';
  } else if (host.lastBackupState) {
    return host.lastBackupState;
  } else if (host.lastBackup?.completed) {
    return 'failed';
  } else {
    return 'idle';
  }
}

export function getColor(state: string) {
  switch (state) {
    case 'waiting':
      return currentTheme.value.warning;
    case 'active':
      return currentTheme.value.info;
    case 'failed':
      return currentTheme.value.error;
    case 'completed':
      return currentTheme.value.success;
    case 'delayed':
      return currentTheme.value.warning;
    case 'disabled':
      return currentTheme.value.secondary;
    case 'idle':
      return currentTheme.value.primary;
  }
}

export function getAvailabilityColor(availibilityState: HostAvailibilityState | undefined | null) {
  switch (availibilityState) {
    case HostAvailibilityState.Online:
      return currentTheme.value.success;
    case HostAvailibilityState.Offline:
      return currentTheme.value.error;
    case HostAvailibilityState.Unknown:
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

export function toDateTime(value: string | number) {
  return format(value, 'MM/dd/yyyy HH:mm');
}

export function toDate(value: number) {
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
