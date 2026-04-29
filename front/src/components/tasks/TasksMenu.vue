<template>
  <v-sheet rounded="lg">
    <v-list rounded="lg">
      <v-list-item link to="/tasks/started">
        <v-list-item-title>Running</v-list-item-title>
        <template v-slot:append>
          <v-badge color="info" :content="stats.running" inline></v-badge>
        </template>
      </v-list-item>
      <v-list-item link to="/tasks/completed">
        <v-list-item-title>Success</v-list-item-title>
        <template v-slot:append>
          <v-badge color="success" :content="stats.success" inline></v-badge>
        </template>
      </v-list-item>
      <v-list-item link to="/tasks/failed">
        <v-list-item-title>Failed</v-list-item-title>
        <template v-slot:append>
          <v-badge color="error" :content="stats.failed" inline></v-badge>
        </template>
      </v-list-item>
      <v-list-item link to="/tasks/created">
        <v-list-item-title>Pending</v-list-item-title>
        <template v-slot:append>
          <v-badge color="warning" :content="stats.pending" inline></v-badge>
        </template>
      </v-list-item>
    </v-list>
  </v-sheet>
</template>

<script lang="ts" setup>
import type { QueueStats } from './stats.interface';

defineProps<{
  stats: QueueStats;
}>();
</script>
