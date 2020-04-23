<script lang="ts">
import { Pie, mixins } from "vue-chartjs";
import { Component, Mixins } from "vue-property-decorator";
const { reactiveData } = mixins;

@Component({})
export default class RepartitionChart extends Mixins(Pie, reactiveData) {
  data = {
    labels: ["pc-ulrich", "pc-eve", "server"],
    datasets: [
      {
        backgroundColor: ["red", "green", "blue"],
        data: [230, 210, 300]
      }
    ]
  };
  options = {
    responsive: true,
    maintainAspectRatio: false
  };
  mounted() {
    this.renderChart(this.data, this.options);
  }
}
</script>
