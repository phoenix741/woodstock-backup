<template>
  <v-timeline-item :dot-color="eventStatusColor" size="small">
    <template v-slot:opposite>
      {{ toDateTime(event.startDate) }} {{ event.endDate && `- ${toDateTime(event.endDate)}` }}
    </template>
    <v-card
      :prepend-icon="icon"
      :append-icon="show ? 'mdi-chevron-up' : 'mdi-chevron-down'"
      class="mx-auto"
      :width="344"
      :subtitle="subtitle"
      :title="title"
      @click="show = !show"
    >
      <v-card-text v-if="show">
        <div class="d-flex flex-wrap">
          <v-chip
            v-if="event.status && event.status !== EventStatus.None"
            :color="eventStatusColor"
            class="ma-2"
            label
            >{{ event.status }}</v-chip
          >
          <v-chip v-if="event.source" class="ma-2" label>
            <v-icon icon="mdi-target" start></v-icon>{{ event.source }}
          </v-chip>
          <v-chip v-if="startDate" class="ma-2" label>
            <v-icon icon="mdi-calendar-start" start></v-icon>{{ startDate }}
          </v-chip>
          <v-chip v-if="executionTime" class="ma-2" label>
            <v-icon icon="mdi-timer" start></v-icon>{{ executionTime }}
          </v-chip>

          <EventBackupInformationComponent
            v-if="event.information?.__typename === 'EventBackupInformation'"
            :information="event.information"
          ></EventBackupInformationComponent>
          <EventRefCountInformationComponent
            v-else-if="event.information?.__typename === 'EventRefCountInformation'"
            :information="event.information"
          ></EventRefCountInformationComponent>
          <EventPoolInformationComponent
            v-else-if="event.information?.__typename === 'EventPoolInformation'"
            :information="event.information"
          ></EventPoolInformationComponent>
          <EventPoolCleanedInformationComponent
            v-else-if="event.information?.__typename === 'EventPoolCleanedInformation'"
            :information="event.information"
          ></EventPoolCleanedInformationComponent>
        </div>

        <template v-if="event.errorMessages?.length">
          <v-alert type="error" dense>
            <v-row v-for="error in event.errorMessages" :key="error">
              <v-col>
                {{ error }}
              </v-col>
            </v-row>
          </v-alert>
        </template>
      </v-card-text>
      <v-card-actions v-if="shoudFix">
        <v-btn color="teal-accent-4" text="Fix" variant="text" @click="launchFix()"></v-btn>
      </v-card-actions>
    </v-card>
  </v-timeline-item>
</template>

<script setup lang="ts">
import { toDateTime } from '@/components/hosts/hosts.utils';
import {
  EventBackupInformation,
  EventPoolCleanedInformation,
  EventPoolInformation,
  EventRefCountInformation,
  EventStatus,
  EventType,
} from '@/generated/graphql';
import { computed, ref } from 'vue';
import { MergedApplicationEvent } from './events.model';
import EventBackupInformationComponent from './EventBackupInformationComponent.vue';
import EventRefCountInformationComponent from './EventRefCountInformationComponent.vue';
import EventPoolInformationComponent from './EventPoolInformationComponent.vue';
import EventPoolCleanedInformationComponent from './EventPoolCleanedInformationComponent.vue';
import filesize from '@/utils/filesize';
import { FormatDistanceFn, FormatDistanceToken, formatDuration, intervalToDuration } from 'date-fns';
import { usePool } from '@/utils/pool';

const { fsckPool } = usePool();

const props = defineProps<{ event: MergedApplicationEvent }>();

const show = ref(false);

const icon = computed(() => {
  switch (props.event.type) {
    case EventType.Backup:
    case EventType.Delete:
    case EventType.Restore:
      return `mdi-server`;

    case EventType.ChecksumChecked:
    case EventType.PoolChecked:
    case EventType.RefcntChecked:
      if (props.event.endDate) {
        return `mdi-check`;
      }
      return `mdi-refresh`;
    case EventType.PoolCleaned:
      if (props.event.endDate) {
        return `mdi-delete`;
      }
      return `mdi-delete-clock`;
  }
});

const formatDistanceLocale: Record<FormatDistanceToken, string> = {
  xSeconds: '{{count}} s',
  xMinutes: '{{count}} m',
  xHours: '{{count}} h',
  lessThanXSeconds: '< {{count}} s',
  halfAMinute: '30 s',
  lessThanXMinutes: '< {{count}} m',
  aboutXHours: '≈ {{count}} h',
  xDays: '{{count}} d',
  aboutXWeeks: '≈ {{count}} w',
  xWeeks: '{{count}} w',
  aboutXMonths: '≈ {{count}} mo',
  xMonths: '{{count}} mo',
  aboutXYears: '≈ {{count}} y',
  xYears: '{{count}} y',
  overXYears: '> {{count}} y',
  almostXYears: '≈ {{count}} y',
};
const shortDistance: FormatDistanceFn = (token, count) =>
  formatDistanceLocale[token].replace('{{count}}', count.toString());

const executionTime = computed(() => {
  if (props.event.endDate) {
    const endDate = new Date(props.event.endDate);
    const startDate = new Date(props.event.startDate);
    const duration = intervalToDuration({ start: startDate, end: endDate });
    return formatDuration(duration, { locale: { formatDistance: shortDistance } });
  }
  return undefined;
});

const startDate = computed(() => {
  return toDateTime(props.event.startDate);
});

const eventStatusColor = computed(() => {
  switch (props.event.status) {
    case EventStatus.Success:
      return 'success';
    case EventStatus.ClientDisconnected:
    case EventStatus.GenericError:
    case EventStatus.ServerCrashed:
      return 'error';
    case EventStatus.None:
      return '';
  }
});

const title = computed(() => {
  switch (props.event.type) {
    case EventType.Backup:
      return `Backup initiated`;
    case EventType.Delete:
      return `Backup removed`;
    case EventType.Restore:
      return `Backup restored`;

    case EventType.ChecksumChecked:
      if (props.event.endDate) {
        return `Checksum completed`;
      }
      return `Checksum initiated`;
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
    case EventType.RefcntChecked:
      if (props.event.endDate) {
        return `Reference count completed`;
      }
      return `Reference count initiated`;
    default:
      return `Event of type ${props.event.type}`;
  }
});

const subtitle = computed(() => {
  switch (props.event?.information?.__typename) {
    case 'EventBackupInformation':
      const backupInformation = props.event.information as EventBackupInformation;
      return `${backupInformation?.hostname} - ${backupInformation?.number}`;
    case 'EventPoolInformation':
      const poolInformation = props.event.information as EventPoolInformation;
      const errorCount = poolInformation?.inNothing + poolInformation?.missing;
      const poolFixed = poolInformation?.fix;
      if (errorCount === 0) {
        return 'No errors found';
      }
      return `${errorCount} errors ${poolFixed ? 'fixed' : 'found'}`;
    case 'EventPoolCleanedInformation':
      const poolCleanedInformation = props.event.information as EventPoolCleanedInformation;
      const size = filesize(poolCleanedInformation?.size);
      return `${size} cleaned`;
    case 'EventRefCountInformation':
      const refCountInformation = props.event.information as EventRefCountInformation;
      const refcountFix = refCountInformation?.fix;
      if (refCountInformation.error === 0) {
        return 'No errors found';
      }
      return `${refCountInformation.error} errors ${refcountFix ? 'fixed' : 'found'}`;
  }
});

const shoudFix = computed(() => {
  switch (props.event?.information?.__typename) {
    case 'EventPoolInformation':
      const poolInformation = props.event.information as EventPoolInformation;
      return !poolInformation?.fix && poolInformation?.missing + poolInformation?.inNothing > 0;
    case 'EventRefCountInformation':
      const refCountInformation = props.event.information as EventRefCountInformation;
      return !refCountInformation?.fix && refCountInformation?.error > 0;
    default:
      return false;
  }
});

async function launchFix() {
  switch (props.event?.type) {
    case EventType.PoolChecked:
    case EventType.RefcntChecked:
      await fsckPool({ fix: true });
  }
}
</script>
