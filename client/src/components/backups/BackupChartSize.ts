import { Pie, mixins } from 'vue-chartjs';
import { Component, Mixins, Prop } from 'vue-property-decorator';
import { Backup } from '@/generated/graphql';
const { reactiveData } = mixins;

@Component({})
export default class BackupChartSize extends Mixins(Pie, reactiveData) {
  @Prop({ required: true })
  backup?: Backup;

  options = {
    responsive: true,
    maintainAspectRatio: false,
  };

  mounted() {
    const data = {
      labels: ['Existing', 'New'],
      datasets: [
        {
          backgroundColor: ['red', 'green'],
          data: [this.backup?.existingFileSize || 0, this.backup?.newFileSize || 0],
        },
      ],
    };
    this.renderChart(data, this.options);
  }
}
