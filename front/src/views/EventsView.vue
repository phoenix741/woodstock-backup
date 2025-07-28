<template>
  <v-container>
    <v-row dense>
      <v-col cols="12" md="6">
        <v-date-input label="First event date" prepend-icon="" prepend-inner-icon="$calendar" variant="solo"
          v-model="startDate"></v-date-input>
      </v-col>

      <v-col cols="12" md="6">
        <v-date-input label="Last event date" prepend-icon="" prepend-inner-icon="$calendar" variant="solo"
          v-model="endDate"></v-date-input>
      </v-col>
    </v-row>
    <v-timeline :hide-opposite="mobile" :density="mobile ? 'compact' : 'default'" side="end">
      <Event v-for="event in mergedEvents" :key="event.uuid" :event="event"></Event>
    </v-timeline>
    <div class="d-flex justify-center align-center mt-4 ga-2">
      <v-btn :disabled="page <= 1" variant="tonal" icon="mdi-chevron-left" @click="page--"></v-btn>
      <span class="text-body-2">Page {{ page }}</span>
      <v-btn :disabled="mergedEvents.length < pageSize" variant="tonal" icon="mdi-chevron-right" @click="page++"></v-btn>
    </div>
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
import { computed, ref, watch } from 'vue';
import { useDisplay } from 'vuetify';
import { VDateInput } from 'vuetify/labs/VDateInput';

const startDate = ref(addMonths(new Date(), -1));
const endDate = ref(new Date());
const page = ref(1);

// Reset to page 1 when the date range changes
watch([startDate, endDate], () => { page.value = 1; });

const { events, pageSize } = useEvents(startDate, endDate, page);
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
            e.information = event.information;
            e.status = event.status;
            e.errorMessages = event.errorMessages;
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
