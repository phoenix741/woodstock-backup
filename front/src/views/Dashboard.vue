<template>
  <v-container>
    <v-row>
      <v-col cols="12" sm="3" v-if="diskUsageStats && diskUsageStats.currentSpace">
        <SpaceUsageCard
          :used="diskUsageStats.currentSpace.used"
          :total="diskUsageStats.currentSpace.size"
          :shared="sharedSize"
        ></SpaceUsageCard>
      </v-col>

      <v-col cols="12" sm="3" v-if="queueStats">
        <QueueRunning :value="queueStats.active" :lastExecution="queueStats.lastExecution"></QueueRunning>
      </v-col>

      <v-col cols="12" sm="3" v-if="queueStats">
        <QueueWaiting :value="queueStats.waiting" :nextWakeup="queueStats.nextWakeup"></QueueWaiting>
      </v-col>

      <v-col cols="12" sm="3" v-if="queueStats">
        <QueueFailed :value="queueStats.failed" :nextWakeup="queueStats.nextWakeup"></QueueFailed>
      </v-col>
    </v-row>

    <v-row>
      <v-col cols="12" sm="6" v-if="diskUsageStats && diskUsageStats.quotas && diskUsageStats.quotas.length">
        <SpaceUsageGraphCard :quotas="diskUsageStats.quotas"></SpaceUsageGraphCard>
      </v-col>
      <v-col
        cols="12"
        sm="6"
        v-if="diskUsageStats && diskUsageStats.compressionStats && diskUsageStats.compressionStats.length"
      >
        <CompressionGraphCard :compressionStats="diskUsageStats.compressionStats"></CompressionGraphCard>
      </v-col>
    </v-row>

    <v-row>
      <v-col cols="12" sm="6" v-if="diskUsageStats && diskUsageStats.quotas && diskUsageStats.quotas.length">
        <CumulatedSpaceUsageGraphCard :quotas="diskUsageStats.quotas"></CumulatedSpaceUsageGraphCard>
      </v-col>
      <v-col cols="12" sm="6" v-if="diskUsageStats && diskUsageStats.quotas && diskUsageStats.quotas.length">
        <RepartitionChartCard :currentRepartition="diskUsageStats.currentRepartition"></RepartitionChartCard>
      </v-col>
    </v-row>
  </v-container>
</template>

<script lang="ts">
import { Component, Vue } from 'vue-property-decorator';
import QueueRunning from '@/components/dashboard/queue/QueueRunning.vue';
import QueueWaiting from '@/components/dashboard/queue/QueueWaiting.vue';
import QueueFailed from '@/components/dashboard/queue/QueueFailed.vue';
import SpaceUsageCard from '@/components/dashboard/cards/SpaceUsageCard.vue';
import CumulatedSpaceUsageGraphCard from '@/components/dashboard/charts/cumulated-space-usage/CumulatedSpaceUsageGraphCard.vue';
import SpaceUsageGraphCard from '@/components/dashboard/charts/space-usage/SpaceUsageGraphCard.vue';
import CompressionGraphCard from '@/components/dashboard/charts/compression/CompressionGraphCard.vue';
import RepartitionChartCard from '@/components/dashboard/charts/repartition/RepartitionChartCard.vue';

import GraphQLDashboardQuery from './Dashboard.graphql';
import { DashboardQuery } from '@/generated/graphql';

@Component({
  components: {
    QueueRunning,
    QueueWaiting,
    QueueFailed,
    SpaceUsageCard,
    SpaceUsageGraphCard,
    CumulatedSpaceUsageGraphCard,
    CompressionGraphCard,
    RepartitionChartCard,
  },
  apollo: {
    queueStats: GraphQLDashboardQuery,
    diskUsageStats: GraphQLDashboardQuery,
  },
})
export default class Dashboard extends Vue {
  queueStats?: DashboardQuery['queueStats'];
  diskUsageStats?: DashboardQuery['diskUsageStats'];

  get sharedSize() {
    const quotas = this.diskUsageStats?.quotas || [];

    return quotas.length && quotas[quotas.length - 1].total.refr;
  }
}
</script>

<style scoped>
.small {
  max-width: 600px;
  height: 200px;
}
</style>
