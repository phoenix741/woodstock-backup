<script lang="ts">
import { Line } from 'vue-chartjs';
import { Component, Mixins, Prop } from 'vue-property-decorator';
import { DashboardQuery } from '../generated/graphql';
import { format } from 'date-fns';
import filesize from 'filesize.js';

@Component({})
export default class SpaceUsageGraph extends Mixins(Line) {
  @Prop()
  quotas?: DashboardQuery['diskUsageStats']['quotas'];

  options = {
    responsive: true,
    maintainAspectRatio: false,
    scales: {
      yAxes: [
        {
          ticks: {
            fontSize: 10,
            callback: (value: number) => filesize(value),
          },
        },
      ],
    },
    tooltips: {
      callbacks: {
        label: function (
          tooltipItem: { datasetIndex: number; yLabel: number },
          data: { datasets: [{ label: string }] },
        ) {
          var label = data.datasets[tooltipItem.datasetIndex].label || '';

          if (label) {
            label += ': ';
          }
          label += filesize(tooltipItem.yLabel);
          return label;
        },
      },
    },
  };

  mounted() {
    const data = {
      labels: this.quotas?.map((quota) => quota.timestamp).map((value) => format(value, 'MM/dd/yyyy')) || [],
      datasets: [
        {
          label: 'Shared',
          backgroundColor: '#3F51B5',
          data: this.quotas?.map((quota) => quota.total.refr),
        },
        {
          label: 'Exclusive',
          backgroundColor: '#9FA8DA',
          stacked: true,
          data: this.quotas?.map((quota) => quota.total.excl),
        },
        {
          label: 'Cumulated Size',
          borderColor: '#F44336',
          fill: false,
          data: this.quotas?.map((quota) => quota.total.total),
        },
      ],
    };
    this.renderChart(data, this.options);
  }
}
</script>
