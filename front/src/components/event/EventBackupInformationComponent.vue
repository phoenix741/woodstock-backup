<template>
  <v-chip v-for="share of information.sharePath" class="ma-2" label
    ><v-icon icon="mdi-share-variant" start></v-icon>{{ share }}</v-chip
  >
</template>

<script setup lang="ts">
import { EventBackupInformation } from '@/generated/graphql';

defineProps<{ information: EventBackupInformation }>();
</script>
