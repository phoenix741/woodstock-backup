<template>
  <v-container>
    <v-row>
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
  </v-container>
</template>

<script lang="ts">
import { Component, Vue } from 'vue-property-decorator';
import QueueRunning from '@/components/dashboard/queue/QueueRunning.vue';
import QueueWaiting from '@/components/dashboard/queue/QueueWaiting.vue';
import QueueFailed from '@/components/dashboard/queue/QueueFailed.vue';

import GraphQLDashboardQuery from './Dashboard.graphql';
import { DashboardQuery } from '@/generated/graphql';

@Component({
  components: {
    QueueRunning,
    QueueWaiting,
    QueueFailed,
  },
  apollo: {
    queueStats: GraphQLDashboardQuery,
    diskUsageStats: GraphQLDashboardQuery,
  },
})
export default class Dashboard extends Vue {
  queueStats?: DashboardQuery['queueStats'];
}
</script>

<style scoped>
.small {
  max-width: 600px;
  height: 200px;
}
</style>
