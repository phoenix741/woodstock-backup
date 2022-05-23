<script setup lang="ts">
import { computed } from "vue";
import { formatDate } from "../../../../filters/formatDate";
import QueueInformation from "./QueueInformation.vue";

const props = defineProps<{
  value: number;
  lastExecution: number;
}>();

const lastExecutionDate = computed(() => {
  return formatDate(props.lastExecution);
});
</script>

<template>
  <QueueInformation
    color="orange"
    icon="mdi-cloud-download"
    label="Running"
    :value="value"
    >Last execution: {{ lastExecutionDate }}</QueueInformation
  >
</template>
