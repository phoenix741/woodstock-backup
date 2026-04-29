<script setup lang="ts">
import filesize from '@/utils/filesize';
import { TreemapChart } from 'echarts/charts';
import { use } from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';
import type { ECBasicOption } from 'echarts/types/dist/shared';
import { computed } from 'vue';
import VChart from 'vue-echarts';
import type { HostBySize } from '../hosts.interface';

use([CanvasRenderer, TreemapChart]);

const props = defineProps<{
  hosts: HostBySize[];
}>();

const option = computed(
  () =>
    ({
      label: {
        position: 'insideTopLeft',
        formatter: function (params: { data: { name: string; originalValue: bigint } }) {
          const arr = [
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
    }) satisfies ECBasicOption,
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
