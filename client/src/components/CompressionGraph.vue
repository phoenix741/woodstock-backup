<script lang="ts">
import { Line } from 'vue-chartjs';
import { Component, Mixins, Prop } from 'vue-property-decorator';
import { DashboardQuery } from '../generated/graphql';
import { format } from 'date-fns';
import filesize from 'filesize.js';

@Component({})
export default class CompressionGraph extends Mixins(Line) {
  @Prop()
  compressionStats?: DashboardQuery['diskUsageStats']['compressionStats'];

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
      labels: this.compressionStats?.map((stat) => stat.timestamp).map((value) => format(value, 'MM/dd/yyyy')) || [],
      datasets: [
        {
          label: 'Disk Usage',
          backgroundColor: '#3F51B5',
          data: this.compressionStats?.map((stat) => stat.diskUsage),
        },
        {
          label: 'Uncompressed',
          borderColor: '#F44336',
          fill: false,
          data: this.compressionStats?.map((stat) => stat.uncompressed),
        },
      ],
    };
    this.renderChart(data, this.options);
  }
}
</script>
