<script lang="ts">
import { Line } from 'vue-chartjs';
import { Component, Mixins, Prop } from 'vue-property-decorator';
import { DashboardQuery } from '../generated/graphql';
import filesize from 'filesize.js';
import { ChartOptions, ChartData } from 'chart.js';

@Component({})
export default class SpaceUsageGraph extends Mixins(Line) {
  @Prop()
  quotas?: DashboardQuery['diskUsageStats']['quotas'];

  mounted() {
    const maxValue = this.quotas?.reduce((acc, value) => {
      if (acc < value.total.total) {
        acc = value.total.total;
      }
      if (acc < value.total.refr + value.total.excl) {
        acc = value.total.refr + value.total.excl;
      }
      return acc;
    }, 0);

    const data: ChartData = {
      labels: this.quotas?.map((quota) => quota.timestamp) || [],
      datasets: [
        {
          label: 'Shared',
          yAxisID: 'stacked',
          backgroundColor: '#3F51B5',
          data: this.quotas?.map((quota) => quota.total.refr),
        },
        {
          label: 'Exclusive',
          yAxisID: 'stacked',
          backgroundColor: '#9FA8DA',
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
            stacked: false,
            ticks: {
              beginAtZero: true,
              fontSize: 10,
              min: 0,
              max: maxValue,
              callback: (value: number) => filesize(value),
            },
          },
          {
            id: 'stacked',
            stacked: true,
            display: false,
            ticks: {
              beginAtZero: true,
              fontSize: 10,
              min: 0,
              max: maxValue,
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
