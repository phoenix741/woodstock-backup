<script lang="ts">
import { Pie, mixins } from 'vue-chartjs';
import { Component, Mixins, Prop } from 'vue-property-decorator';
import { DashboardQuery } from '../generated/graphql';
const { reactiveData } = mixins;

@Component({})
export default class RepartitionChart extends Mixins(Pie, reactiveData) {
  @Prop()
  lastQuota?: DashboardQuery['diskUsageStats']['quotas'][0];

  options = {
    responsive: true,
    maintainAspectRatio: false,
  };
  mounted() {
    const data = {
      labels: this.lastQuota?.host?.map((h) => h.host) || [],
      datasets: [
        {
          backgroundColor: [
            '#F44336',
            '#E91E63',
            '#9C27B0',
            '#673AB7',
            '#3F51B5',
            '#2196F3',
            '#03A9F4',
            '#00BCD4',
            '#009688',
            '#4CAF50',
            '#8BC34A',
            '#CDDC39',
            '#FFEB3B',
            '#FFC107',
            '#FF9800',
            '#FF5722',
            '#795548',
            '#607D8B',
            '#9E9E9E',
          ],
          data: this.lastQuota?.host?.map((h) => h.total),
        },
      ],
    };
    this.renderChart(data, this.options);
  }
}
</script>
