<template>
  <QueueInformation color="orange" icon="mdi-cloud-download" label="Running" :value="value"
    >Last execution: {{ lastExecution | date }}</QueueInformation
  >
</template>

<script lang="ts">
import { Component, Vue, Prop } from 'vue-property-decorator';
import QueueInformation from './QueueInformation.vue';

@Component({
  components: { QueueInformation },
})
export default class QueueRunning extends Vue {
  @Prop({ required: true, type: Number })
  value!: number;

  @Prop({ required: true, type: Number })
  lastExecution!: number;
}
</script>
