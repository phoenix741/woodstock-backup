<script setup lang="ts">
import { formatFilesize } from "@/filters/formatFilesize";
import { formatNumber } from "@/filters/formatNumber";
import type { DashboardQuery } from "@/graphql";
import { computed } from "@vue/reactivity";
import { LineChart } from "echarts/charts";
import { GridComponent } from "echarts/components";
import { use } from "echarts/core";
import { CanvasRenderer } from "echarts/renderers";
import VChart from "vue-echarts";

use([CanvasRenderer, GridComponent, LineChart]);

const props = defineProps<{
  pool: DashboardQuery["statistics"]["poolUsage"];
}>();

const colors = ["#5470C6", "#EE6666"];
const option = computed(() => ({
  color: colors,
  xAxis: [
    {
      type: "time",
      axisLabel: {
        rotate: 45,
      },
    },
  ],
  yAxis: [
    {
      type: "value",
      name: "Size",
      position: "left",
      alignTicks: true,
      axisLine: {
        show: true,
        lineStyle: {
          color: colors[0],
        },
      },
      axisLabel: {
        formatter: (f: number) => formatFilesize(f * 1024 * 1024),
      },
    },
    {
      type: "value",
      name: "Count",
      position: "right",
      alignTicks: true,
      axisLine: {
        show: true,
        lineStyle: {
          color: colors[1],
        },
      },
      axisLabel: {
        formatter: (f: number) => formatNumber(f),
      },
    },
  ],
  series: [
    {
      name: "Compressed Pool usage",
      type: "line",
      yAxisIndex: 0,
      smooth: true,
      symbol: "none",
      data: props.pool?.compressedSizeRange?.map(({ time, value }) => [
        Number(time),
        value,
      ]),
    },
    {
      name: "Nb Chunks",
      type: "line",
      yAxisIndex: 1,
      lineStyle: {
        type: "dashed",
      },
      data: props.pool?.nbChunkRange?.map(({ time, value }) => [
        Number(time),
        value,
      ]),
    },
  ],
}));
</script>

<template>
  <v-card class="mx-auto ma-5">
    <v-card-title class="text-overline">Pool Usage</v-card-title>
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
