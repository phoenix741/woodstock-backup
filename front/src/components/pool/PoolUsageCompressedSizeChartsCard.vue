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
import { toDate } from '@/components/hosts/hosts.utils';
import { LineChart } from 'echarts/charts';
import { GridComponent } from 'echarts/components';
import { use } from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';
import { computed } from 'vue';
import VChart from 'vue-echarts';
import filesize from '../../utils/filesize';
import type { CompressedSizePoolUsage } from './pool.interface';

use([CanvasRenderer, GridComponent, LineChart]);

const props = defineProps<{
  compressedSizeRange: CompressedSizePoolUsage[];
}>();

const option = computed(() => ({
  xAxis: [
    {
      type: 'time',
      axisLabel: {
        rotate: 45,
        formatter: (value: string | number) => toDate(value),
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
      data: props.compressedSizeRange.map(({ time, value }) => [time, Number(value / 1024n / 1024n)]),
    },
  ],
}));
</script>

<style scoped>
.chart {
  height: 300px;
}
</style>
