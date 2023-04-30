<template>
  <v-card class="mx-auto">
    <v-card-title class="text-overline">{{ title }}</v-card-title>

    <v-list-item class="pb-3">
      <template v-slot:prepend>
        <v-avatar :color="color">
          <v-icon x-large>{{ icon }}</v-icon>
        </v-avatar>
      </template>

      <v-list-item-title>{{ filesize(used) }} / {{ filesize(total) }}</v-list-item-title>
      <v-list-item-subtitle v-if="total">
        <v-progress-linear
          :color="color"
          :model-value="Number((100n * (used - (buffer ?? 0n))) / total)"
          :buffer-value="Number((100n * used) / total)"
        ></v-progress-linear>
      </v-list-item-subtitle>
    </v-list-item>

    <v-divider></v-divider>
    <v-card-actions>
      <v-icon class="mr-2" small :color="usedPercentColor">{{ iconHistory }}</v-icon>
      <span class="text-caption font-weight-light">
        <span :class="'text-' + usedPercentColor" v-html="usedPercentString"></span> Since last month
      </span>
    </v-card-actions>
  </v-card>
</template>

<script lang="ts" setup>
import filesize from '@/utils/filesize';
import { computed } from 'vue';
import { toPercent } from '../hosts/hosts.utils';

const props = defineProps<{
  title: string;
  icon: string;
  color: string;
  used: bigint;
  total: bigint;
  buffer?: bigint;
  yesterday?: bigint;
}>();

const usedPercent = computed(() => {
  const yesterday = props.yesterday ?? props.used ?? 0n;
  const percent = props.used ? ((props.used - yesterday) * 100n) / props.used : 100n;
  return Number(percent);
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
