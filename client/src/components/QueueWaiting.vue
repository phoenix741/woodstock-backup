<template>
  <QueueInformation color="green" icon="mdi-animation" label="Queue" :value="value"
    >Next wakeup: {{ nextWakeup | date }}</QueueInformation
  >
</template>

<script lang="ts">
import { Component, Vue, Prop } from 'vue-property-decorator';
import QueueInformation from './QueueInformation.vue';

@Component({
  components: { QueueInformation },
})
export default class QueueWaiting extends Vue {
  @Prop({ required: true, type: Number })
  value!: number;

  @Prop({ required: true, type: Number })
  nextWakeup!: number;
}
</script>
