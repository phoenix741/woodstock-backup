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
      <v-col cols="12" sm="6" v-if="diskUsageStats && diskUsageStats.quotas">
        <SpaceUsageGraphCard :quotas="diskUsageStats.quotas"></SpaceUsageGraphCard>
      </v-col>
      <v-col cols="12" sm="6" v-if="diskUsageStats && diskUsageStats.compressionStats">
        <CompressionGraphCard :compressionStats="diskUsageStats.compressionStats"></CompressionGraphCard>
      </v-col>
    </v-row>

    <v-row>
      <v-col cols="12" sm="12" v-if="diskUsageStats && diskUsageStats.quotas">
        <RepartitionChartCard :currentRepartition="diskUsageStats.currentRepartition"></RepartitionChartCard>
      </v-col>
    </v-row>
  </v-container>
</template>

<script lang="ts">
import { Component, Vue } from 'vue-property-decorator';
import RepartitionChart from '../components/RepartitionChart.vue';
import QueueRunning from '../components/QueueRunning.vue';
import QueueWaiting from '../components/QueueWaiting.vue';
import QueueFailed from '../components/QueueFailed.vue';
import SpaceUsageCard from '../components/SpaceUsageCard.vue';
import SpaceUsageGraphCard from '../components/SpaceUsageGraphCard.vue';
import CompressionGraphCard from '../components/CompressionGraphCard.vue';
import RepartitionChartCard from '../components/RepartitionChartCard.vue';

import GraphQLDashboardQuery from './Dashboard.graphql';
import { DashboardQuery } from '../generated/graphql';

@Component({
  components: {
    RepartitionChart,
    QueueRunning,
    QueueWaiting,
    QueueFailed,
    SpaceUsageCard,
    SpaceUsageGraphCard,
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
