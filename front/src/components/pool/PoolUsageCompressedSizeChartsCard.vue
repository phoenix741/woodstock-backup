<template>
  <v-card class="mx-auto">
    <v-card-title class="text-overline">Compression size Pool Usage</v-card-title>
    <v-card-text>
      <v-sheet class="mx-auto">
        <v-chart class="chart" :option="option" :autoresize="true" />
      </v-sheet>
    </v-card-text>
  </v-card>
</template>

<script setup lang="ts">
import { LineChart } from 'echarts/charts';
import { GridComponent } from 'echarts/components';
import { use } from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';
import { computed, provide } from 'vue';
import VChart, { THEME_KEY } from 'vue-echarts';
import filesize from '../../utils/filesize';
import { CompressedSizePoolUsage } from './pool.interface';

use([CanvasRenderer, GridComponent, LineChart]);

provide(THEME_KEY, 'dark');

const props = defineProps<{
  compressedSizeRange: CompressedSizePoolUsage[];
}>();

const colors = ['#5470C6', '#EE6666'];
const option = computed(() => ({
  color: colors,
  xAxis: [
    {
      type: 'time',
      axisLabel: {
        rotate: 45,
      },
    },
  ],
  yAxis: [
    {
      type: 'value',
      name: 'Compressed size',
      position: 'left',
      alignTicks: true,
      axisLine: {
        show: true,
        lineStyle: {
          color: colors[0],
        },
      },
      axisLabel: {
        formatter: (f: number) => filesize(BigInt(f) * 1024n * 1024n),
      },
    },
  ],
  series: [
    {
      name: 'Compressed Pool usage',
      type: 'line',
      yAxisIndex: 0,
      smooth: true,
      symbol: 'none',
      areaStyle: {},
      data: props.compressedSizeRange.map(({ time, value }) => [time * 1000, Number(value / 1024n / 1024n)]),
    },
  ],
}));
</script>

<style scoped>
.chart {
  height: 300px;
}
</style>
