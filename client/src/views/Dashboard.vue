<template>
  <v-container>
    <v-row>
      <v-col cols="12" sm="3">
        <v-card class="mx-auto" flat>
          <v-list-item two-line>
            <v-list-item-avatar tile size="80">
              <v-icon x-large color="blue">mdi-harddisk</v-icon>
            </v-list-item-avatar>

            <v-list-item-content>
              <div class="overline mb-4">Used space</div>
              <v-list-item-title class="headline mb-1">650 Go / 3000 Go</v-list-item-title>
              <v-list-item-subtitle>
                <v-progress-linear value="15"></v-progress-linear>
              </v-list-item-subtitle>
            </v-list-item-content>
          </v-list-item>

          <v-divider></v-divider>
          <v-card-text>
            <v-icon class="mr-2" small>
              mdi-clock-outline
            </v-icon>
            <span class="caption grey--text font-weight-light"> Shared space: 250 Go</span>
          </v-card-text>
        </v-card>
      </v-col>

      <v-col cols="12" sm="3">
        <QueueRunning :value="queueStats.active" :lastExecution="queueStats.lastExecution"></QueueRunning>
      </v-col>

      <v-col cols="12" sm="3">
        <QueueWaiting :value="queueStats.waiting" :nextWakeup="queueStats.nextWakeup"></QueueWaiting>
      </v-col>

      <v-col cols="12" sm="3">
        <QueueFailed :value="queueStats.failed" :nextWakeup="queueStats.nextWakeup"></QueueFailed>
      </v-col>
    </v-row>

    <v-row>
      <v-col cols="12" sm="4">
        <v-card class="mx-auto" flat>
          <v-card-title class="overline mb-4">Usage</v-card-title>
          <v-card-text>
            <v-sheet class="mx-auto" max-width="calc(100% - 32px)">
              <!-- CHART JS -->
              <StorageChart styles="height: 250px"></StorageChart>
            </v-sheet>
          </v-card-text>

          <v-simple-table>
            <template v-slot:default>
              <thead>
                <tr>
                  <th class="text-left">Name</th>
                  <th class="text-left">Size</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td>Backup Size</td>
                  <td>230 Go</td>
                </tr>
                <tr>
                  <td>Shared</td>
                  <td>210 Go</td>
                </tr>
                <tr>
                  <td>Exclusive</td>
                  <td>10 Go</td>
                </tr>
              </tbody>
            </template>
          </v-simple-table>
        </v-card>
      </v-col>
      <v-col cols="12" sm="4">
        <v-card class="mx-auto" flat>
          <v-card-title class="overline mb-4">Compression</v-card-title>
          <v-card-text>
            <v-sheet class="mx-auto" max-width="calc(100% - 32px)">
              <!-- CHART JS -->
              <StorageChart styles="height: 250px"></StorageChart>
            </v-sheet>
          </v-card-text>

          <v-simple-table>
            <template v-slot:default>
              <thead>
                <tr>
                  <th class="text-left">Name</th>
                  <th class="text-left">Size</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td>Total</td>
                  <td>230 Go</td>
                </tr>
                <tr>
                  <td>Compressed</td>
                  <td>210 Go</td>
                </tr>
              </tbody>
            </template>
          </v-simple-table>
        </v-card>
      </v-col>
      <v-col cols="12" sm="4">
        <v-card class="mx-auto" flat>
          <v-card-title class="overline mb-4">Répartition</v-card-title>
          <v-card-text>
            <v-sheet class="mx-auto" max-width="calc(100% - 32px)">
              <!-- CHART JS -->
              <RepartitionChart styles="height: 250px"></RepartitionChart>
            </v-sheet>
          </v-card-text>

          <v-simple-table>
            <template v-slot:default>
              <thead>
                <tr>
                  <th class="text-left">Name</th>
                  <th class="text-left">Size</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td>pc-ulrich</td>
                  <td>230 Go</td>
                </tr>
                <tr>
                  <td>pc-eve</td>
                  <td>210 Go</td>
                </tr>
                <tr>
                  <td>server</td>
                  <td>300 Go</td>
                </tr>
              </tbody>
            </template>
          </v-simple-table>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script lang="ts">
import { Component, Vue } from 'vue-property-decorator';
import StorageChart from '../components/StorageChart.vue';
import RepartitionChart from '../components/RepartitionChart.vue';
import QueueRunning from '../components/QueueRunning.vue';
import QueueWaiting from '../components/QueueWaiting.vue';
import QueueFailed from '../components/QueueFailed.vue';

import GraphQLDashboardQuery from './Dashboard.graphql';
import { DashboardQuery } from '../generated/graphql';

@Component({
  components: { StorageChart, RepartitionChart, QueueRunning, QueueWaiting, QueueFailed },
  apollo: {
    queueStats: GraphQLDashboardQuery,
  },
})
export default class Dashboard extends Vue {
  queueStats!: DashboardQuery['queueStats'];
}
</script>

<style scoped>
.small {
  max-width: 600px;
  height: 200px;
}
</style>
