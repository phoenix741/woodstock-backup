<template>
  <v-card class="mx-auto" flat>
    <v-card-title class="overline mb-4">Répartition</v-card-title>
    <v-card-text>
      <v-sheet class="mx-auto" max-width="calc(100% - 32px)">
        <!-- CHART JS -->
        <RepartitionChart :currentRepartition="currentRepartition" class="chart"></RepartitionChart>
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
        <tbody v-if="currentRepartition">
          <tr v-for="host in currentRepartition" :key="host.host">
            <td>{{ host.host }}</td>
            <td>{{ host.total | filesize }}</td>
          </tr>
        </tbody>
      </template>
    </v-simple-table>
  </v-card>
</template>

<script lang="ts">
import { Vue, Component, Prop } from 'vue-property-decorator';
import { DashboardQuery } from '@/generated/graphql';
import RepartitionChart from './RepartitionChart.vue';

@Component({
  components: {
    RepartitionChart,
  },
})
export default class RepartitionChartCard extends Vue {
  @Prop()
  currentRepartition?: DashboardQuery['diskUsageStats']['currentRepartition'];
}
</script>

<style scoped>
.chart {
  height: 250px;
}
</style>
