<script lang="ts">
import { Line } from 'vue-chartjs';
import { Component, Mixins, Prop } from 'vue-property-decorator';
import { DashboardQuery } from '@/generated/graphql';
import { ChartOptions } from 'chart.js';
import numeral from 'numeral';

@Component({})
export default class CompressionGraph extends Mixins(Line) {
  @Prop()
  compressionStats?: DashboardQuery['diskUsageStats']['compressionStats'];

  options: ChartOptions = {
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
            callback: (value: number) => numeral(value).format('0.00 %'),
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
          label += numeral(tooltipItem.yLabel).format('0.00 %');
          return label;
        },
      },
    },
  };

  mounted() {
    const data = {
      labels: this.compressionStats?.map((stat) => stat.timestamp) || [],
      datasets: [
        {
          label: 'Compression',
          borderColor: '#3F51B5',
          fill: false,
          data: this.compressionStats?.map((stat) => stat.diskUsage / stat.uncompressed),
        },
      ],
    };
    this.renderChart(data, this.options);
  }
}
</script>
