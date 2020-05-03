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

  @Prop()
  hostname?: string;
  @Prop()
  number?: number;

  content = '';

  async mounted() {
    await this.fetchData();
  }

  @Watch('$route')
  async fetchData() {
    let url;
    if (this.hostname) {
      url = `/api/hosts/${this.hostname}/backups/${this.number}/log/${this.logfile}?tailable=false`;
    } else {
      url = `/api/server/log/${this.logfile}?tailable=false`;
    }
    const response = await axios(url);
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
