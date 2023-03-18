<script lang="ts" setup>
import { ref } from 'vue';

defineProps<{
  url: string;
}>();

const logframe = ref(null);
</script>

<template>
  <iframe ref="logframe" :src="url"></iframe>
</template>

<style scoped>
iframe {
  border: 0;
  height: 500px;
  width: 100%;
}
</style>
