<template>
  <v-card class="mx-auto">
    <v-card-title class="text-overline">{{ title }}</v-card-title>

    <v-list-item class="pb-3">
      <template v-slot:prepend>
        <v-avatar :color="color">
          <v-icon x-large>{{ icon }}</v-icon>
        </v-avatar>
      </template>

      <v-list-item-title>{{ usedString }}</v-list-item-title>
    </v-list-item>

    <v-divider></v-divider>
    <v-card-actions>
      <v-icon class="mr-2" small :color="usedPercentColor">{{ iconHistory }}</v-icon>
      <span class="text-caption text-grey font-weight-light">
        <span :class="'text-' + usedPercentColor" v-html="usedPercentString"></span>
        Since last month
      </span>
    </v-card-actions>
  </v-card>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { toNumber, toPercent } from '../hosts/hosts.utils';

const props = withDefaults(
  defineProps<{
    title: string;
    color: string;
    icon: string;
    used?: number;
    yesterday?: number;
  }>(),
  {
    used: 0,
    total: 0,
    yesterday: 0,
  },
);

const usedString = computed(() => toNumber(props.used));

const usedPercent = computed(() => {
  const yesterday = props.yesterday ?? props.used ?? 0;
  const percent = ((props.used - yesterday) / props.used) * 100;
  return percent;
});

const usedPercentString = computed(() => toPercent(usedPercent.value));

const usedPercentColor = computed(() => (usedPercent.value > 0 ? 'red' : 'green'));

const iconHistory = computed(() =>
  usedPercent.value > 0
    ? 'mdi-arrow-top-right'
    : usedPercent.value === 0
    ? 'mdi-arrow-right'
    : 'mdi-arrow-bottom-right',
);
</script>
