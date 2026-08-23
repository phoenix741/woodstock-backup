<template>
  <v-container>
    <v-row dense>
      <v-col cols="12" md="6">
        <v-date-input
          label="First event date"
          prepend-icon=""
          prepend-inner-icon="$calendar"
          variant="solo"
          v-model="startDate"
        ></v-date-input>
      </v-col>

      <v-col cols="12" md="6">
        <v-date-input
          label="Last event date"
          prepend-icon=""
          prepend-inner-icon="$calendar"
          variant="solo"
          v-model="endDate"
        ></v-date-input>
      </v-col>
    </v-row>
    <v-row dense>
      <v-col cols="6" md="3">
        <v-select
          label="Type"
          prepend-icon=""
          prepend-inner-icon="mdi-shape-outline"
          variant="solo"
          :items="eventTypeOptions"
          v-model="eventType"
          clearable
        ></v-select>
      </v-col>
      <v-col cols="6" md="3">
        <v-select
          label="Status"
          prepend-icon=""
          prepend-inner-icon="mdi-check-circle-outline"
          variant="solo"
          :items="eventStatusOptions"
          v-model="eventStatus"
          clearable
        ></v-select>
      </v-col>
      <v-col cols="6" md="3">
        <v-select
          label="Source"
          prepend-icon=""
          prepend-inner-icon="mdi-target"
          variant="solo"
          :items="eventSourceOptions"
          v-model="eventSource"
          clearable
        ></v-select>
      </v-col>
      <v-col cols="6" md="3">
        <v-autocomplete
          label="Host"
          prepend-icon=""
          prepend-inner-icon="mdi-server"
          variant="solo"
          :items="hostnameOptions"
          v-model="hostname"
          clearable
        ></v-autocomplete>
      </v-col>
    </v-row>

    <v-sheet rounded="lg" border>
      <v-list density="compact" lines="two">
        <template v-for="item in eventsWithDayLabel" :key="item.event.uuid">
          <v-list-subheader v-if="item.dayLabel">{{ item.dayLabel }}</v-list-subheader>
          <Event :event="item.event"></Event>
        </template>
      </v-list>
    </v-sheet>

    <div class="d-flex justify-center align-center mt-4 ga-2">
      <v-btn :disabled="page <= 1" variant="tonal" icon="mdi-chevron-left" @click="page--"></v-btn>
      <span class="text-body-2">Page {{ toNumber(page) }} / {{ toNumber(totalPages) }}</span>
      <v-btn :disabled="page >= totalPages" variant="tonal" icon="mdi-chevron-right" @click="page++"></v-btn>
    </div>
  </v-container>
</template>

<script setup lang="ts">
import Event from '@/components/event/EventComponent.vue';
import { MergedApplicationEventFragment } from '@/components/event/events.fragment';
import { eventSourceOptions, eventStatusOptions, eventTypeOptions } from '@/components/event/events.labels';
import type { MergedApplicationEvent } from '@/components/event/events.model';
import { toDate, toNumber } from '@/components/hosts/hosts.utils';
import { useFragment } from '@/generated';
import { EventSource, EventStatus, EventType, type EventsFilterInput } from '@/generated/graphql';
import { useDevices } from '@/utils/devices';
import { useEvents } from '@/utils/events';
import { addMonths } from 'date-fns';
import { computed, ref, watch } from 'vue';
import { VDateInput } from 'vuetify/labs/VDateInput';

const startDate = ref(addMonths(new Date(), -1));
const endDate = ref(new Date());
const page = ref(1);

const eventType = ref<EventType>();
const eventStatus = ref<EventStatus>();
const eventSource = ref<EventSource>();
const hostname = ref<string>();

const filter = computed<EventsFilterInput | undefined>(() => {
  if (!eventType.value && !eventStatus.value && !eventSource.value && !hostname.value) {
    return undefined;
  }
  return {
    type: eventType.value,
    status: eventStatus.value,
    source: eventSource.value,
    hostname: hostname.value,
  };
});

// Reset to page 1 when the date range or filters change
watch([startDate, endDate, filter], () => {
  page.value = 1;
});

const { events, totalCount, pageSize } = useEvents(startDate, endDate, page, filter);
const { devices } = useDevices();

const hostnameOptions = computed(() => devices.value?.hosts.map((host) => host.name) ?? []);

const totalPages = computed(() => Math.max(1, Math.ceil(totalCount.value / pageSize)));

const mergedEvents = computed<Array<MergedApplicationEvent>>(
  () => events.value?.map((event) => useFragment(MergedApplicationEventFragment, event)) ?? [],
);

// Server already returns events sorted and paginated; this only tags the first event of
// each day so a v-list-subheader can be inserted before it.
const eventsWithDayLabel = computed(() => {
  let previousDay: string | undefined;
  return mergedEvents.value.map((event) => {
    const reference = event.endDate ?? event.startDate;
    const day = reference ? new Date(reference).toDateString() : 'unknown';
    const dayLabel = day === previousDay ? undefined : reference ? toDate(reference) : 'Unknown date';
    previousDay = day;
    return { event, dayLabel };
  });
});
</script>
