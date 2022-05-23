<script setup lang="ts">
import { computed } from "vue";
import { formatDate } from "../../../../filters/formatDate";
import QueueInformation from "./QueueInformation.vue";

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
    color="green"
    icon="mdi-animation"
    label="Queue"
    :value="value"
    >Next wakeup: {{ nextWakeupDate }}</QueueInformation
  >
</template>
