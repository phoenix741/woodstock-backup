<template>
  <v-chip class="ma-2" label><v-icon icon="mdi-dots-grid" start></v-icon>{{ information.count }}</v-chip>
  <v-chip class="ma-2" label><v-icon icon="mdi-space-invaders" start></v-icon>{{ information.error }}</v-chip>
</template>

<script setup lang="ts">
import { EventRefCountInformation } from '@/generated/graphql';

defineProps<{ information: EventRefCountInformation }>();
</script>
