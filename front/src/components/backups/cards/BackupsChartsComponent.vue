<script setup lang="ts">
import { toDate } from '@/components/hosts/hosts.utils';
import filesize from '@/utils/filesize';
import { BarChart, LineChart } from 'echarts/charts';
import { GridComponent } from 'echarts/components';
import { use } from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';
import { computed, provide } from 'vue';
import VChart, { THEME_KEY } from 'vue-echarts';
import { BackupSizeByDate } from '../backups.interface';

const BIGINT_DELTA = 1024n * 1024n;

use([CanvasRenderer, GridComponent, BarChart, LineChart]);

provide(THEME_KEY, 'dark');

const props = defineProps<{
  backups: BackupSizeByDate[];
}>();

const option = computed(() => ({
  xAxis: [
    {
      type: 'category',
      axisLabel: {
        rotate: 45,
        formatter: (f: string) => toDate(parseInt(f)),
      },
    },
  ],
  yAxis: [
    {
      type: 'value',
      name: 'Total File Size',
      smooth: true,
      axisLabel: {
        formatter: (f: number) => filesize(BigInt(f) * BIGINT_DELTA),
      },
    },
    {
      type: 'value',
      name: 'New File Size',
      smooth: true,
      axisLabel: {
        formatter: (f: number) => filesize(BigInt(f) * BIGINT_DELTA),
      },
    },
  ],
  series: [
    {
      name: 'Total File Size',
      type: 'bar',
      data: props.backups.map(({ startDate, fileSize }) => [startDate, Number(fileSize / BIGINT_DELTA)]),
    },
    {
      name: 'New File Size',
      type: 'line',
      yAxisIndex: 1,
      data: props.backups.map(({ startDate, newFileSize }) => [startDate, Number(newFileSize / BIGINT_DELTA)]),
    },
  ],
}));
</script>

<template>
  <v-card class="mx-auto">
    <v-card-title class="text-overline">Backups history</v-card-title>
    <v-card-text>
      <v-sheet class="mx-auto">
        <v-chart class="chart" :option="option" :autoresize="true" />
      </v-sheet>
    </v-card-text>
  </v-card>
</template>

<style scoped>
.chart {
  height: 400px;
}
</style>
