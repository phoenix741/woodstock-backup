<script setup lang="ts">
import type { DashboardQuery } from "@/graphql";
import { computed } from "@vue/reactivity";
import { TreemapChart } from "echarts/charts";
import { use } from "echarts/core";
import { CanvasRenderer } from "echarts/renderers";
import VChart from "vue-echarts";
import { formatFilesize } from "../../../../filters/formatFilesize";

use([CanvasRenderer, TreemapChart]);

const props = defineProps<{
  hosts: DashboardQuery["statistics"]["hosts"];
}>();

const option = computed(() => ({
  label: {
    position: "insideTopLeft",
    formatter: function (params: { name: string; value: number }) {
      let arr = [
        "{name|" + params.name + "}",
        "{hr|}",
        "{budget| " + formatFilesize(params.value * 1024 * 1024) + "}",
      ];

      return arr.join("\n");
    },
    rich: {
      budget: {
        fontSize: 22,
        lineHeight: 30,
        color: "yellow",
      },
      name: {
        fontSize: 12,
        color: "#fff",
      },
      hr: {
        width: "100%",
        borderColor: "rgba(255,255,255,0.2)",
        borderWidth: 0.5,
        height: 0,
        lineHeight: 10,
      },
    },
  },
  series: [
    {
      type: "treemap",
      breadcrumb: {
        show: false,
      },

      nodeClick: undefined,
      data: props.hosts?.map((v) => ({
        name: v.host,
        value: v.compressedSize,
      })),
    },
  ],
}));
</script>

<template>
  <v-card class="mx-auto ma-5">
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
