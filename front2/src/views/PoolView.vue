<script setup lang="ts">
import { useQuery } from "villus";
import SpaceUsageCard from "../components/metrics/cards/space/SpaceUsageCard.vue";
import CounterUsageCard from "../components/metrics/cards/space/CounterUsageCard.vue";
import PoolUsageChartsComponentVue from "@/components/metrics/cards/pool/PoolUsageChartsComponent.vue";
import { DashboardDocument } from "../graphql";

const { data } = useQuery({
  query: DashboardDocument,
});
</script>

<template>
  <v-container>
    <v-row>
      <v-col cols="3">
        <SpaceUsageCard
          title="Used space"
          icon="mdi-harddisk"
          color="blue"
          :used="data?.statistics?.diskUsage?.used || undefined"
          :total="data?.statistics?.diskUsage?.total || undefined"
          :yesterday="data?.statistics?.diskUsage?.usedLastMonth || undefined"
        ></SpaceUsageCard>
      </v-col>
      <v-col cols="3" class="d-flex flex-wrap">
        <SpaceUsageCard
          title="Pool space"
          icon="mdi-zip-box"
          color="pink"
          :used="data?.statistics?.poolUsage?.compressedSize || undefined"
          :total="data?.statistics?.poolUsage?.size || undefined"
          :yesterday="
            data?.statistics?.poolUsage?.compressedSizeLastMonth || undefined
          "
        ></SpaceUsageCard>
      </v-col>
      <v-col cols="3">
        <CounterUsageCard
          title="Chunks"
          icon="mdi-checkerboard"
          color="amber"
          :used="data?.statistics?.poolUsage?.nbChunk || undefined"
          :yesterday="
            data?.statistics?.poolUsage?.nbChunkLastMonth || undefined
          "
        ></CounterUsageCard>
      </v-col>
      <v-col cols="3">
        <CounterUsageCard
          title="References"
          icon="mdi-dots-grid"
          color="purple"
          :used="data?.statistics?.poolUsage?.nbRef || undefined"
          :yesterday="data?.statistics?.poolUsage?.nbRefLastMonth || undefined"
        ></CounterUsageCard>
      </v-col>
    </v-row>
    <v-row>
      <v-col cols="12">
        <PoolUsageChartsComponentVue
          v-if="data?.statistics.poolUsage?.compressedSizeRange?.length"
          :pool="data?.statistics.poolUsage || {}"
        ></PoolUsageChartsComponentVue>
      </v-col>
    </v-row>
  </v-container>
</template>
