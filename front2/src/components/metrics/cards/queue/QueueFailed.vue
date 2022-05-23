<script setup lang="ts">
import { computed } from "vue";
import QueueInformation from "./QueueInformation.vue";
import { formatDate } from "../../../../filters/formatDate";

const props = defineProps<{
  value: number;
  nextWakeup: number;
}>();

const nextWakeupDate = computed(() => {
  return formatDate(props.nextWakeup);
});
</script>

<template>
  <QueueInformation
    color="red"
    icon="mdi-alert-circle-outline"
    label="Failed"
    :value="value"
    >Next wakeup: {{ nextWakeupDate }}</QueueInformation
  >
</template>
