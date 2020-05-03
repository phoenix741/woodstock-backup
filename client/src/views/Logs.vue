<template>
  <pre>{{ content }}</pre>
</template>

<script lang="ts">
import { Component, Vue, Prop, Watch } from 'vue-property-decorator';
import axios from 'axios';

@Component({})
export default class Logs extends Vue {
  @Prop()
  logfile!: string;

  content = '';

  async mounted() {
    await this.fetchData();
  }

  @Watch('$route')
  async fetchData() {
    const response = await axios(`/api/server/log/${this.logfile}?tailable=false`);
    this.content = response.data;
    Vue.nextTick(() => {
      window.scrollTo(0, document.body.scrollHeight);
    });
  }
}
</script>

<style scoped>
pre {
  height: 100%;
  overflow-x: auto;
  font-size: 8pt;
}
</style>
