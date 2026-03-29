<template>
  <v-timeline-item :dot-color="eventStatusColor" size="small">
    <template v-slot:opposite>
      {{ startDate }}
      {{ event.endDate && `- ${toDateTime(event.endDate)}` }}
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
          <v-chip title="Source" v-if="event.source" class="ma-2" label>
            <v-icon icon="mdi-target" start></v-icon>{{ event.source }}
          </v-chip>
          <v-chip title="Start date" v-if="startDate" class="ma-2" label>
            <v-icon icon="mdi-calendar-start" start></v-icon>{{ startDate }}
          </v-chip>
          <v-chip title="Execution time" v-if="executionTime" class="ma-2" label>
            <v-icon icon="mdi-timer" start></v-icon>{{ executionTime }}
          </v-chip>

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
import { parseDateTime, toDateTime, toNumber } from '@/components/hosts/hosts.utils';
import { useFragment } from '@/generated';
import { EventStatus, EventType } from '@/generated/graphql';
import filesize from '@/utils/filesize';
import { formatDurationValue } from '@/utils/formatting';
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
import type { MergedApplicationEvent } from './events.model';

const { fsckPool } = usePool();

const props = defineProps<{ event: MergedApplicationEvent }>();

const show = ref(false);

const icon = computed(() => {
  switch (props.event.type) {
    case EventType.Backup:
    case EventType.Delete:
    case EventType.Restore:
      return `mdi-server`;

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
  if (props.event.endDate) {
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

const eventStatusColor = computed(() => {
  switch (props.event.status) {
    case EventStatus.Success:
      return 'success';
    case EventStatus.ClientDisconnected:
    case EventStatus.GenericError:
    case EventStatus.ServerCrashed:
      return 'error';
    default:
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
