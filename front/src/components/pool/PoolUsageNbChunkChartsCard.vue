<template>
  <v-card class="mx-auto">
    <v-card-title class="text-overline">Pool Usage Nb chunk</v-card-title>
    <v-card-text>
      <v-chart class="chart" :option="option" :autoresize="true" />
    </v-card-text>
  </v-card>
</template>

<script setup lang="ts">
import { format } from 'date-fns';
import { HeatmapChart } from 'echarts/charts';
import { CalendarComponent, TooltipComponent, VisualMapComponent } from 'echarts/components';
import { use } from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';
import { computed, provide } from 'vue';
import VChart, { THEME_KEY } from 'vue-echarts';
import { NbChunkPoolUsage } from './pool.interface';

provide(THEME_KEY, 'dark');

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
  const nbChunkRange = props.nbChunkRange.map(({ time, ...rest }) => ({ time: time * 1000, ...rest }));

  const array = [...emptyChunk.value, ...nbChunkRange]
    .sort((a, b) => a.time - b.time)
    .map(({ time, value }) => ({ time: format(time, 'yyyy-MM-dd'), value }))
    .reduce((acc, { time, value }, currentIndex, array) => {
      const previous =
        currentIndex > 0 ? array.findLast((val, i) => !!val.value && i <= currentIndex - 1)?.value ?? 0 : 0;
      acc[time] = value ?? previous;
      return acc;
    }, {} as Record<string, number>);

  const entries = Object.entries(array).map(([time, value], i, array) => {
    const previous = i > 0 ? array[i - 1] : [0, 0];
    return [time, value - previous[1]] as [string, number]; // , value - previous[1]
  });

  return entries;
});

const minValue = computed(() => {
  const minValue = Math.min(...nbChunkRange.value.map((item) => item[1]));
  const minValueStr = Math.abs(minValue).toString();
  const size = minValueStr.length;
  const abs = minValue < 0 ? -1 : 1;

  return abs * parseInt(minValueStr[0] + '0'.repeat(size - 1)) - parseInt(1 + '0'.repeat(size - 1));
});

const maxValue = computed(() => {
  const maxValue = Math.max(...nbChunkRange.value.map((item) => item[1]));
  const maxValueStr = Math.abs(maxValue).toString();
  const size = maxValueStr.length;
  const abs = maxValue < 0 ? -1 : 1;

  return abs * parseInt(maxValueStr[0] + '0'.repeat(size - 1)) + parseInt(1 + '0'.repeat(size - 1));
});

const option = computed(() => ({
  tooltip: {},
  visualMap: {
    type: 'continuous',
    min: minValue.value,
    max: maxValue.value,
    calculable: true,
    orient: 'horizontal',
    left: 'center',
    top: 10,
    width: 500,
    inRange: {
      color: ['#EDE7F6', '#D1C4E9', '#B39DDB', '#9575CD', '#7E57C2', '#5E35B1', '#512DA8', '#4527A0', '#311B92'],
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
    data: nbChunkRange.value,
  },
}));
</script>

<style scoped>
.chart {
  height: 200px;
}
</style>
