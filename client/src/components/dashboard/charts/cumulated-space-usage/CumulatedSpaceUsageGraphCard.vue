<template>
  <v-card class="mx-auto" flat>
    <v-card-title class="overline mb-4">Cumulated Usage</v-card-title>
    <v-card-text>
      <v-sheet class="mx-auto" max-width="calc(100% - 32px)">
        <!-- CHART JS -->
        <CumulatedSpaceUsageGraph :quotas="quotas" class="chart"></CumulatedSpaceUsageGraph>
      </v-sheet>
    </v-card-text>

    <v-simple-table v-if="lastQuota">
      <template v-slot:default>
        <thead>
          <tr>
            <th class="text-left">Name</th>
            <th class="text-left">Size</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>Cumulated Size</td>
            <td>{{ lastQuota.total.total | filesize }}</td>
          </tr>
        </tbody>
      </template>
    </v-simple-table>
  </v-card>
</template>

<script lang="ts">
import { Component, Vue, Prop } from 'vue-property-decorator';
import { DashboardQuery } from '@/generated/graphql';
import CumulatedSpaceUsageGraph from './CumulatedSpaceUsageGraph.vue';

@Component({
  components: {
    CumulatedSpaceUsageGraph,
  },
})
export default class CumulatedSpaceUsageGraphCard extends Vue {
  @Prop()
  quotas?: DashboardQuery['diskUsageStats']['quotas'];

  get lastQuota() {
    return this.quotas?.length && this.quotas[this.quotas.length - 1];
  }
}
</script>

<style scoped>
.chart {
  height: 250px;
}
</style>
