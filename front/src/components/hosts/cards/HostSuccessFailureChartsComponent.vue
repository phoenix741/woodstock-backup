<script setup lang="ts">
import { PieChart } from 'echarts/charts';
import { LegendComponent, TooltipComponent } from 'echarts/components';
import { use } from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';
import { computed, provide } from 'vue';
import VChart, { THEME_KEY } from 'vue-echarts';
import { HostCountByState } from '../hosts.interface';
import { getColor } from '../hosts.utils';

use([CanvasRenderer, LegendComponent, TooltipComponent, PieChart]);

provide(THEME_KEY, 'dark');

const props = defineProps<{
  countByState: HostCountByState[];
}>();

const option = computed(() => ({
  //   color: ["blue", "red", "green", "yellow"],
  tooltip: {
    trigger: 'item',
  },
  legend: {
    top: '5%',
    left: 'center',
  },
  series: [
    {
      name: 'countByState',
      type: 'pie',
      radius: ['40%', '70%'],
      avoidLabelOverlap: false,
      label: {
        show: false,
        position: 'center',
      },
      emphasis: {
        label: {
          show: true,
          fontSize: '40',
          fontWeight: 'bold',
        },
      },
      labelLine: {
        show: false,
      },
      data: props.countByState.map((v) => ({
        ...v,
        itemStyle: { color: getColor(v.name) },
      })),
    },
  ],
}));
</script>

<template>
  <v-card class="mx-auto">
    <v-card-title class="text-overline">Success / Failure</v-card-title>
    <v-card-text>
      <v-sheet class="mx-auto">
        <v-chart class="chart" :option="option" :autoresize="true" />
      </v-sheet>
    </v-card-text>
  </v-card>
</template>

<style scoped>
.chart {
  height: 300px;
}
</style>
