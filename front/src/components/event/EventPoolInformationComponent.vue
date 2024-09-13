<template>
  <v-chip class="ma-2" label><v-icon icon="mdi-dots-grid" start></v-icon>{{ information.inRefcnt }}</v-chip>
  <v-chip class="ma-2" label><v-icon icon="mdi-delete" start></v-icon>{{ information.inUnused }}</v-chip>
  <v-chip class="ma-2" label><v-icon icon="mdi-space-invaders" start></v-icon>{{ information.inNothing }}</v-chip>
  <v-chip class="ma-2" label><v-icon icon="mdi-balloon" start></v-icon>{{ information.missing }}</v-chip>
</template>

<script setup lang="ts">
import { EventPoolInformation } from '@/generated/graphql';

defineProps<{ information: EventPoolInformation }>();
</script>
