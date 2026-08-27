<template>
  <div class="event-row" :class="{ 'event-row--open': show }">
    <div class="event-row__main" @click="show = !show">
      <v-icon :icon="icon" :color="eventStatusColor || undefined" size="small" class="event-row__icon"></v-icon>
      <span class="event-row__time text-medium-emphasis">{{ timeLabel }}</span>
      <div class="event-row__title">
        <div class="text-body-2 font-weight-medium">{{ title }}</div>
        <div class="text-caption text-medium-emphasis event-row__subtitle">
          <router-link v-if="backupLink" :to="backupLink" @click.stop>{{ subtitle }}</router-link>
          <span v-else>{{ subtitle }}</span>
        </div>
      </div>
      <v-chip v-if="executionTime" size="small" label variant="tonal" class="event-row__duration">{{
        executionTime
      }}</v-chip>
      <v-chip v-if="statusChip" :color="statusChip.color" size="small" label variant="tonal" class="event-row__chip">{{
        statusChip.text
      }}</v-chip>
      <v-icon size="small" class="text-medium-emphasis event-row__caret">{{
        show ? 'mdi-chevron-up' : 'mdi-chevron-down'
      }}</v-icon>
    </div>

    <v-expand-transition>
      <div v-if="show" class="event-row__detail">
        <v-card variant="tonal" rounded="lg" class="event-row__detail-card">
          <v-table density="compact" class="bg-transparent">
            <tbody>
              <tr v-if="event.source">
                <td>Source</td>
                <td class="text-right">{{ eventSourceLabel(event.source) }}</td>
              </tr>
              <tr v-if="startDate">
                <td>Start date</td>
                <td class="text-right">{{ startDate }}</td>
              </tr>
            </tbody>
          </v-table>
        </v-card>

        <EventBackupInformationComponent
          v-if="event.information?.__typename === 'EventBackupInformation'"
          :information="event.information"
        ></EventBackupInformationComponent>
        <EventPoolInformationComponent
          v-else-if="event.information?.__typename === 'EventPoolInformation'"
          :information="event.information"
        ></EventPoolInformationComponent>
        <EventPoolCleanedInformationComponent
          v-else-if="event.information?.__typename === 'EventPoolCleanedInformation'"
          :information="event.information"
        ></EventPoolCleanedInformationComponent>
        <EventHashConversionComponent
          v-else-if="event.information?.__typename === 'EventHashConversionInformation'"
          :information="event.information"
        ></EventHashConversionComponent>

        <template v-if="event.errorMessages?.length">
          <v-alert type="error" dense class="mt-2">
            <v-row v-for="error in event.errorMessages" :key="error">
              <v-col>
                {{ error }}
              </v-col>
            </v-row>
          </v-alert>
        </template>

        <div v-if="shoudFix" class="mt-1">
          <v-btn color="teal-accent-4" text="Fix" variant="text" size="small" @click="launchFix()"></v-btn>
        </div>
      </div>
    </v-expand-transition>
  </div>
</template>

<script setup lang="ts">
import { parseDateTime, toDateTime, toNumber } from '@/components/hosts/hosts.utils';
import { useFragment } from '@/generated';
import { EventStatus, EventType } from '@/generated/graphql';
import filesize from '@/utils/filesize';
import { formatDateValue, formatDurationValue } from '@/utils/formatting';
import { usePool } from '@/utils/pool';
import { computed, ref } from 'vue';
import EventBackupInformationComponent from './EventBackupInformationComponent.vue';
import EventPoolCleanedInformationComponent from './EventPoolCleanedInformationComponent.vue';
import EventPoolInformationComponent from './EventPoolInformationComponent.vue';
import EventHashConversionComponent from './EventHashConversionComponent.vue';
import {
  EventBackupInformationFragment,
  EventPoolCleanedInformationFragment,
  EventPoolInformationFragment,
  EventHashConversionInformationFragment,
} from './events.fragment';
import { eventSourceLabel, eventStatusLabel } from './events.labels';
import type { MergedApplicationEvent } from './events.model';

const { fsckPool } = usePool();

const props = defineProps<{ event: MergedApplicationEvent }>();

const show = ref(false);

// Types that never emit an End row (logged once, after the fact, always terminal).
// Can't be inferred from `status === None`: a genuinely in-progress Start/End type
// (Backup, PoolChecked, ...) also has `status === None` while running — that's the
// exact ambiguity that caused the original bug. Only the event TYPE tells you whether
// an End row will ever arrive, so a new single-shot type must be added here explicitly.
const SINGLE_SHOT_EVENT_TYPES: EventType[] = [EventType.Delete];
const isSingleShot = computed(() => SINGLE_SHOT_EVENT_TYPES.includes(props.event.type));

const icon = computed(() => {
  switch (props.event.type) {
    case EventType.Backup:
    case EventType.Restore:
      return `mdi-server`;
    case EventType.Delete:
      return `mdi-delete`;

    case EventType.PoolChecked:
      if (props.event.endDate) {
        return `mdi-check`;
      }
      return `mdi-refresh`;
    case EventType.PoolCleaned:
      if (props.event.endDate) {
        return `mdi-delete`;
      }
      return `mdi-delete-clock`;
    default:
      return undefined;
  }
});

const executionTime = computed(() => {
  if (props.event.endDate && props.event.startDate) {
    const endDate = parseDateTime(props.event.endDate);
    const startDate = parseDateTime(props.event.startDate);
    return formatDurationValue(endDate.getTime() - startDate.getTime(), {
      unitDisplay: 'short',
      maxParts: 2,
      listStyle: 'short',
    });
  }
  return undefined;
});

const startDate = computed(() => {
  return props.event.startDate ? toDateTime(props.event.startDate) : 'unknown';
});

const timeLabel = computed(() => {
  const reference = props.event.startDate ?? props.event.endDate;
  return reference ? formatDateValue(reference, { timeStyle: 'short' }) : '--:--';
});

const eventStatusColor = computed(() => {
  switch (props.event.status) {
    case EventStatus.Success:
      return 'success';
    case EventStatus.ClientDisconnected:
    case EventStatus.GenericError:
    case EventStatus.ServerCrashed:
    case EventStatus.Aborted:
      return 'error';
    case EventStatus.Cancelled:
      return 'warning';
    default:
      return '';
  }
});

const statusChip = computed(() => {
  if (props.event.status && props.event.status !== EventStatus.None) {
    return { text: eventStatusLabel(props.event.status), color: eventStatusColor.value };
  }
  if (!isSingleShot.value && !props.event.endDate) {
    return { text: 'In progress', color: 'info' };
  }
  return undefined;
});

const title = computed(() => {
  switch (props.event.type) {
    case EventType.Backup:
      return `Backup initiated`;
    case EventType.Delete:
      return `Backup removed`;
    case EventType.Restore:
      return `Backup restored`;

    case EventType.PoolChecked:
      if (props.event.endDate) {
        return `Pool content completed`;
      }
      return `Pool content initiated`;
    case EventType.PoolCleaned:
      if (props.event.endDate) {
        return `Pool cleaning completed`;
      }
      return `Pool cleaning initiated`;
    case EventType.HashConversion:
      if (props.event.endDate) {
        return `Hash conversion completed`;
      }
      return `Hash conversion initiated`;
    default:
      return `Event of type ${props.event.type}`;
  }
});

const backupLink = computed(() => {
  if (props.event?.information?.__typename !== 'EventBackupInformation') {
    return undefined;
  }
  const backupInformation = useFragment(EventBackupInformationFragment, props.event.information);
  if (!backupInformation?.backupId) {
    return undefined;
  }
  return {
    name: 'BackupDetails',
    params: { deviceId: backupInformation.hostname, backupId: backupInformation.backupId },
  };
});

const subtitle = computed(() => {
  switch (props.event?.information?.__typename) {
    case 'EventBackupInformation': {
      const backupInformation = useFragment(EventBackupInformationFragment, props.event.information);
      return `${backupInformation?.hostname} - ${toNumber(backupInformation?.number)}`;
    }
    case 'EventPoolInformation': {
      const poolInformation = useFragment(EventPoolInformationFragment, props.event.information);
      const errorCount =
        poolInformation?.inNothing +
        poolInformation?.missing +
        poolInformation?.refcountError +
        poolInformation?.chunkError;
      const poolFixed = poolInformation?.fix;
      if (errorCount === 0) {
        return 'No errors found';
      }
      return `${toNumber(errorCount)} errors ${poolFixed ? 'fixed' : 'found'}`;
    }
    case 'EventPoolCleanedInformation': {
      const poolCleanedInformation = useFragment(EventPoolCleanedInformationFragment, props.event.information);
      const size = filesize(poolCleanedInformation?.size);
      return `${size} cleaned`;
    }
    case 'EventHashConversionInformation': {
      const hashConversionInformation = useFragment(EventHashConversionInformationFragment, props.event.information);
      return `${toNumber(hashConversionInformation?.count)} hashes converted`;
    }
    default:
      return '';
  }
});

const shoudFix = computed(() => {
  switch (props.event?.information?.__typename) {
    case 'EventPoolInformation': {
      const poolInformation = useFragment(EventPoolInformationFragment, props.event.information);
      return (
        !poolInformation?.fix &&
        poolInformation?.missing + poolInformation?.inNothing + poolInformation?.refcountError > 0
      );
    }
    default:
      return false;
  }
});

async function launchFix() {
  switch (props.event?.type) {
    case EventType.PoolChecked:
      await fsckPool({ fix: true, verifyChunks: false });
  }
}
</script>

<style scoped>
.event-row {
  border-bottom: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
}

.event-row__main {
  display: grid;
  grid-template-columns: 24px 76px 1fr auto auto 20px;
  grid-template-areas: 'icon time title duration chip caret';
  align-items: center;
  gap: 12px;
  padding: 8px 4px;
  cursor: pointer;
}

.event-row__icon {
  grid-area: icon;
  justify-self: center;
}

.event-row__time {
  grid-area: time;
  font-variant-numeric: tabular-nums;
  font-size: 0.75rem;
  white-space: nowrap;
}

.event-row__title {
  grid-area: title;
  min-width: 0;
}

.event-row__subtitle {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.event-row__duration {
  grid-area: duration;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

.event-row__chip {
  grid-area: chip;
  justify-self: end;
}

.event-row__caret {
  grid-area: caret;
}

.event-row__detail {
  padding: 0 4px 12px 60px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.event-row__detail-card {
  max-width: 480px;
}

.event-row__detail :deep(.v-table) {
  --v-table-row-height: 36px;
}

@media (max-width: 600px) {
  .event-row__main {
    grid-template-columns: 24px 1fr auto 20px;
    grid-template-areas: 'icon title chip caret';
  }
  .event-row__time,
  .event-row__duration {
    display: none;
  }
}
</style>
