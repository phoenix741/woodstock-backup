<script setup lang="ts">
import { computed } from "@vue/reactivity";
import { PieChart } from "echarts/charts";
import { LegendComponent, TooltipComponent } from "echarts/components";
import { use } from "echarts/core";
import { CanvasRenderer } from "echarts/renderers";
import VChart from "vue-echarts";
import { getColor } from "../../../../utils/hosts.utils";

use([CanvasRenderer, LegendComponent, TooltipComponent, PieChart]);

const props = defineProps<{
  countByState: Array<{ name: string; value: number }>;
}>();

const option = computed(() => ({
  //   color: ["blue", "red", "green", "yellow"],
  tooltip: {
    trigger: "item",
  },
  legend: {
    top: "5%",
    left: "center",
  },
  series: [
    {
      name: "countByState",
      type: "pie",
      radius: ["40%", "70%"],
      avoidLabelOverlap: false,
      label: {
        show: false,
        position: "center",
      },
      emphasis: {
        label: {
          show: true,
          fontSize: "40",
          fontWeight: "bold",
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
  <v-card class="mx-auto ma-5">
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
