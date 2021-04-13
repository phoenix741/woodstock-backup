<template>
  <v-card class="mx-auto" flat>
    <v-card-title class="overline mb-4">Usage</v-card-title>
    <v-card-text>
      <v-sheet class="mx-auto" max-width="calc(100% - 32px)">
        <!-- CHART JS -->
        <CompressionGraph :compressionStats="compressionStats" class="chart"></CompressionGraph>
      </v-sheet>
    </v-card-text>

    <v-simple-table v-if="lastCompressionStat">
      <template v-slot:default>
        <thead>
          <tr>
            <th class="text-left">Name</th>
            <th class="text-left">Size</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>Compression ratio</td>
            <td>{{ (100 * (lastCompressionStat.diskUsage / lastCompressionStat.uncompressed)) | formatPercent }}</td>
          </tr>
        </tbody>
      </template>
    </v-simple-table>
  </v-card>
</template>

<script lang="ts">
import { Component, Vue, Prop } from 'vue-property-decorator';
import { DashboardQuery } from '@/generated/graphql';
import CompressionGraph from './CompressionGraph.vue';

@Component({
  components: {
    CompressionGraph,
  },
})
export default class CompressionGraphCard extends Vue {
  @Prop()
  compressionStats?: DashboardQuery['diskUsageStats']['compressionStats'];

  get lastCompressionStat() {
    return this.compressionStats?.length && this.compressionStats[this.compressionStats.length - 1];
  }
}
</script>

<style scoped>
.chart {
  height: 250px;
}
</style>
