<template>
  <v-card class="mx-auto" flat>
    <v-card-title class="overline mb-4">Usage</v-card-title>
    <v-card-text>
      <v-sheet class="mx-auto" max-width="calc(100% - 32px)">
        <!-- CHART JS -->
        <SpaceUsageGraph :quotas="quotas" class="chart"></SpaceUsageGraph>
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
            <td>Shared</td>
            <td>{{ lastQuota.total.refr | filesize }}</td>
          </tr>
          <tr>
            <td>Exclusive</td>
            <td>{{ lastQuota.total.excl | filesize }}</td>
          </tr>
        </tbody>
      </template>
    </v-simple-table>
  </v-card>
</template>

<script lang="ts">
import { Component, Vue, Prop } from 'vue-property-decorator';
import { DashboardQuery } from '@/generated/graphql';
import SpaceUsageGraph from './SpaceUsageGraph.vue';

@Component({
  components: {
    SpaceUsageGraph,
  },
})
export default class SpaceUsageGraphCard extends Vue {
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
