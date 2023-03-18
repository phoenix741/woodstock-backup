<script setup lang="ts">
import { computed, provide } from 'vue';
import { TreemapChart } from 'echarts/charts';
import { use } from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';
import VChart, { THEME_KEY } from 'vue-echarts';
import filesize from '@/utils/filesize';
import { HostBySize } from '../hosts.interface';
import type { ECBasicOption } from 'echarts/types/dist/shared';

use([CanvasRenderer, TreemapChart]);

provide(THEME_KEY, 'dark');

const props = defineProps<{
  hosts: HostBySize[];
}>();

const option = computed(
  () =>
    ({
      color: [
        '#311B92',
        '#4527A0',
        '#512DA8',
        '#5E35B1',
        '#6200EA',
        '#651FFF',
        '#7C4DFF',
        '#7E57C2',
        '#9575CD',
        '#B388FF',
        '#B39DDB',
        '#D1C4E9',
        '#EDE7F6',
      ],
      label: {
        position: 'insideTopLeft',
        formatter: function (params: { data: { name: string; originalValue: bigint } }) {
          let arr = [
            '{name|' + params.data.name + '}',
            '{hr|}',
            '{budget| ' + filesize(params.data.originalValue) + '}',
          ];

          return arr.join('\n');
        },
        rich: {
          budget: {
            fontSize: 22,
            lineHeight: 30,
            color: 'yellow',
          },
          name: {
            fontSize: 12,
            color: '#fff',
          },
          hr: {
            width: '100%',
            borderColor: 'rgba(255,255,255,0.2)',
            borderWidth: 0.5,
            height: 0,
            lineHeight: 10,
          },
        },
      },
      series: [
        {
          type: 'treemap',
          breadcrumb: {
            show: false,
          },

          nodeClick: undefined,
          data: (props.hosts || []).map(({ name, value }) => ({
            name,
            value: Number(value / 1024n / 1024n),
            originalValue: value,
          })),
        },
      ],
    } satisfies ECBasicOption),
);
</script>

<template>
  <v-card class="mx-auto">
    <v-card-title class="text-overline">Host repartition</v-card-title>
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
