<template>
  <iframe ref="logframe" :src="url"></iframe>
</template>

<script lang="ts">
import { Component, Vue, Prop, Ref } from 'vue-property-decorator';

@Component({})
export default class Logs extends Vue {
  @Prop()
  logfile!: string;

  @Prop()
  hostname?: string;
  @Prop()
  number?: number;

  @Ref('logframe')
  iframe!: HTMLIFrameElement;

  get url() {
    if (this.hostname) {
      return `/api/hosts/${this.hostname}/backups/${this.number}/log/${this.logfile}?tailable=false`;
    } else {
      return `/api/server/log/${this.logfile}?tailable=false`;
    }
  }
}
</script>

<style scoped>
iframe {
  border: 0;
  height: 100%;
  width: 100%;
}
</style>
