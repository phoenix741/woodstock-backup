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
    <v-timeline :hide-opposite="mobile" :density="mobile ? 'compact' : 'default'" side="end">
      <Event v-for="event in mergedEvents" :key="event.uuid" :event="event"></Event>
    </v-timeline>
  </v-container>
</template>

<script setup lang="ts">
import Event from '@/components/event/EventComponent.vue';
import { ApplicationEventFragment } from '@/components/event/events.fragment';
import type { MergedApplicationEvent } from '@/components/event/events.model';
import { useFragment } from '@/generated';
import { EventStep } from '@/generated/graphql';
import { useEvents } from '@/utils/events';
import { addMonths } from 'date-fns';
import { computed, ref } from 'vue';
import { useDisplay } from 'vuetify';
import { VDateInput } from 'vuetify/labs/VDateInput';

const startDate = ref(addMonths(new Date(), -1));
const endDate = ref(new Date());

const { events } = useEvents(startDate, endDate);
const { mobile } = useDisplay();

const mergedEvents = computed<Array<MergedApplicationEvent>>(() => {
  const mergedEvents =
    events.value?.reduce(
      (acc, eventFragment) => {
        const { timestamp, step, ...event } = useFragment(ApplicationEventFragment, eventFragment);
        const e = acc[event.uuid] ?? { ...event };
        switch (step) {
          case EventStep.Start:
            e.startDate = timestamp;
            break;
          case EventStep.End:
            e.endDate = timestamp;
            break;
        }
        acc[event.uuid] = e;

        return acc;
      },
      {} as Record<string, MergedApplicationEvent>,
    ) ?? {};

  const sortedEvents = Object.values(mergedEvents).sort((a, b) => {
    if (a.startDate && b.startDate) {
      const startDateA = new Date(a.startDate);
      const startDateB = new Date(b.startDate);
      return startDateB.getTime() - startDateA.getTime();
    }
    return 0;
  });

  return sortedEvents;
});
</script>
