<script lang="ts">
import { Line } from 'vue-chartjs';
import { Component, Mixins, Prop } from 'vue-property-decorator';
import { DashboardQuery } from '@/generated/graphql';
import filesize from 'filesize.js';
import { ChartOptions, ChartData } from 'chart.js';

@Component({})
export default class Cumulated extends Mixins(Line) {
  @Prop()
  quotas?: DashboardQuery['diskUsageStats']['quotas'];

  mounted() {
    const data: ChartData = {
      labels: this.quotas?.map((quota) => quota.timestamp) || [],
      datasets: [
        {
          label: 'Cumulated Size',
          borderColor: '#F44336',
          fill: false,
          data: this.quotas?.map((quota) => quota.total.total),
        },
      ],
    };
    const options: ChartOptions = {
      responsive: true,
      maintainAspectRatio: false,
      scales: {
        xAxes: [
          {
            type: 'time',
            distribution: 'linear',
            offset: true,
            ticks: {
              source: 'data',
              maxRotation: 0,
              autoSkip: true,
              autoSkipPadding: 75,
            },
          },
        ],
        yAxes: [
          {
            ticks: {
              beginAtZero: true,
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
    this.renderChart(data, options);
  }
}
</script>
