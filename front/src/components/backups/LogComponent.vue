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
  display: block;
  border: 0;
  height: 450px;
  width: 100%;
}
</style>
