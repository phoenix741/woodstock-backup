<template>
  <v-card class="mx-auto">
    <v-card-title class="text-overline">Pool Usage Nb chunk</v-card-title>
    <v-card-text>
      <v-chart class="chart" :option="option" :autoresize="true" />
    </v-card-text>
  </v-card>
</template>

<script setup lang="ts">
import { toDate, toNumber } from '@/components/hosts/hosts.utils';
import { format } from 'date-fns';
import { HeatmapChart } from 'echarts/charts';
import { CalendarComponent, TooltipComponent, VisualMapComponent } from 'echarts/components';
import { use } from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';
import { computed } from 'vue';
import VChart from 'vue-echarts';
import type { NbChunkPoolUsage } from './pool.interface';

use([CanvasRenderer, VisualMapComponent, TooltipComponent, CalendarComponent, HeatmapChart]);

const props = defineProps<{
  nbChunkRange: NbChunkPoolUsage[];
}>();

const firstDate = new Date().getTime() - 365 * 24 * 3600 * 1000;
const lastDate = new Date().getTime();

const range = [format(firstDate, 'yyyy-MM-dd'), format(lastDate, 'yyyy-MM-dd')];

const emptyChunk = computed(() => {
  const values = [];
  for (let time = firstDate; time < lastDate; time += 24 * 3600 * 1000) {
    values.push({
      time,
      value: undefined,
    });
  }
  return values;
});

const nbChunkRange = computed(() => {
  // Filter chunk that are before firstDate
  const nbChunkRange = props.nbChunkRange.map(({ time, ...rest }) => ({ time: new Date(time).getTime(), ...rest }));

  const array = [...emptyChunk.value, ...nbChunkRange]
    .sort((a, b) => a.time - b.time)
    .map(({ time, value }) => ({ time: format(time, 'yyyy-MM-dd'), value }))
    .reduce(
      (acc, { time, value }, currentIndex, array) => {
        const previous =
          currentIndex > 0 ? (array.findLast((val, i) => !!val.value && i <= currentIndex - 1)?.value ?? 0) : 0;
        acc[time] = value ?? previous;
        return acc;
      },
      {} as Record<string, number>,
    );

  const entries = Object.entries(array).map(([time, value], i, array) => {
    const previous = i > 0 ? array[i - 1] : [0, 0];
    return [time, value - previous[1]] as [string, number]; // , value - previous[1]
  });

  return entries;
});

// Sign-preserving log transform: compresses extreme ranges so small values remain visible.
// e.g. log10(400001) ≈ 5.6, log10(101) ≈ 2.0 → 36% ratio instead of 0.025% with linear scale
const logTransform = (v: number) => (v === 0 ? 0 : Math.sign(v) * Math.log10(Math.abs(v) + 1));
const inverseLogTransform = (v: number) => (v === 0 ? 0 : Math.sign(v) * (Math.pow(10, Math.abs(v)) - 1));

// Include all days (including zero-delta days) to avoid gaps in the heatmap calendar
const nbChunkRangeTransformed = computed(() =>
  nbChunkRange.value.map(([time, value]) => [time, logTransform(value)] as [string, number]),
);

const minTransformed = computed(() => {
  const values = nbChunkRangeTransformed.value.map(([, v]) => v);
  return values.length ? Math.min(...values) : 0;
});

const maxTransformed = computed(() => {
  const values = nbChunkRangeTransformed.value.map(([, v]) => v);
  return values.length ? Math.max(...values) : 1;
});

const option = computed(() => ({
  tooltip: {
    formatter: (params: { data: [string, number] }) => {
      const [date, transformedVal] = params.data;
      const original = Math.round(inverseLogTransform(transformedVal));
      const sign = original > 0 ? '+' : '';
      return `${toDate(date)}<br/>${sign}${toNumber(original)} chunks`;
    },
  },
  visualMap: {
    type: 'continuous',
    min: minTransformed.value,
    max: maxTransformed.value,
    calculable: true,
    orient: 'horizontal',
    left: 'center',
    top: 10,
    width: 500,
    formatter: (v: number) => {
      const original = Math.round(inverseLogTransform(v));
      const sign = original > 0 ? '+' : '';
      return `${sign}${toNumber(original)}`;
    },
  },
  calendar: {
    top: 90,
    left: 30,
    right: 30,
    cellSize: ['auto', 13],
    range,
    itemStyle: {
      borderWidth: 0.5,
    },
    yearLabel: { show: false },
  },
  series: {
    type: 'heatmap',
    coordinateSystem: 'calendar',
    data: nbChunkRangeTransformed.value,
  },
}));
</script>

<style scoped>
.chart {
  height: 200px;
}
</style>
