<script lang="ts">
import { Line, mixins } from "vue-chartjs";
import { Component, Mixins } from "vue-property-decorator";
const { reactiveData } = mixins;

@Component({})
export default class StorageChart extends Mixins(Line, reactiveData) {
  data = {
    labels: [
      "01/01/2020",
      "02/01/2020",
      "03/01/2020",
      "04/01/2020",
      "05/01/2020",
      "06/01/2020",
      "07/01/2020",
      "08/01/2020",
      "09/01/2020",
      "10/01/2020"
    ],
    datasets: [
      {
        label: "Shared",
        backgroundColor: "#318be3",
        data: [
          10000,
          11000,
          11000,
          11000,
          11000,
          12000,
          12000,
          12000,
          12000,
          12000
        ]
      },
      {
        label: "Exclusive",
        backgroundColor: "#04befe",
        stacked: true,
        data: [
          11000,
          13000,
          14000,
          14000,
          14100,
          15200,
          15200,
          15200,
          15200,
          15400
        ]
      },
      {
        label: "Total",
        borderColor: "red",
        fill: false,
        data: [
          20000,
          21000,
          23000,
          23000,
          23100,
          23200,
          23200,
          23200,
          23200,
          23400
        ]
      }
    ]
  };
  options = {
    responsive: true,
    maintainAspectRatio: false,
    scales: {
      yAxes: [
        {
          type: "logarithmic"
        }
      ]
    }
  };
  mounted() {
    this.renderChart(this.data, this.options);
  }
}
</script>
