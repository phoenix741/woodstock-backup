<script setup lang="ts">
import { computed } from "vue";
import { formatFilesize } from "../../../../filters/formatFilesize";
import { formatPercent } from "../../../../filters/formatNumber";

const props = withDefaults(
  defineProps<{
    title: string;
    icon: string;
    color: string;
    used?: number;
    total?: number;
    yesterday?: number;
  }>(),
  {
    used: 0,
    total: 0,
    yesterday: 0,
  }
);

const usedString = computed(() => {
  return formatFilesize(props.used * 1024 * 1024);
});
const totalString = computed(() => {
  return formatFilesize(props.total * 1024 * 1024);
});
const usedPercent = computed(() => {
  const yesterday = props.yesterday || props.used || 0;
  const percent = ((props.used - yesterday) / props.used) * 100;
  return percent;
});
const usedPercentString = computed(() => {
  return formatPercent(usedPercent.value);
});
const usedPercentColor = computed(() => {
  return usedPercent.value > 0 ? "red" : "green";
});
const iconHistory = computed(() => {
  return usedPercent.value > 0
    ? "mdi-arrow-top-right"
    : usedPercent.value === 0
    ? "mdi-arrow-right"
    : "mdi-arrow-bottom-right";
});
</script>

<template>
  <v-card class="mx-auto ma-5" :min-width="280" :max-width="280">
    <v-card-header>
      <v-card-header-text>
        <div class="text-overline">{{ title }}</div>
        <v-list-item-title class="headline"
          >{{ usedString }} / {{ totalString }}</v-list-item-title
        >
        <v-list-item-subtitle>
          <v-progress-linear
            :model-value="(100 * used) / total"
          ></v-progress-linear>
        </v-list-item-subtitle>
      </v-card-header-text>
      <v-card-avatar>
        <v-avatar :color="color">
          <v-icon color="white" :icon="icon"></v-icon>
        </v-avatar>
      </v-card-avatar>
    </v-card-header>

    <v-divider></v-divider>
    <v-card-text>
      <v-icon class="mr-2" small :color="usedPercentColor">{{
        iconHistory
      }}</v-icon>
      <span class="text-caption text-grey font-weight-light">
        <span
          :class="'text-' + usedPercentColor"
          v-html="usedPercentString"
        ></span>
        Since last month
      </span>
    </v-card-text>
  </v-card>
</template>
