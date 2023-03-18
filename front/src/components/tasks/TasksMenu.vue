<template>
  <v-sheet rounded="lg">
    <v-list rounded="lg">
      <v-list-item link to="/tasks/active">
        <v-list-item-title>Active</v-list-item-title>
        <template v-slot:append>
          <v-badge color="info" :content="stats.active" inline></v-badge>
        </template>
      </v-list-item>
      <v-list-item link to="/tasks/completed">
        <v-list-item-title>Success</v-list-item-title>
        <template v-slot:append>
          <v-badge color="success" :content="stats.completed" inline></v-badge>
        </template>
      </v-list-item>
      <v-list-item link to="/tasks/failed">
        <v-list-item-title>Failed</v-list-item-title>
        <template v-slot:append>
          <v-badge color="error" :content="stats.failed" inline></v-badge>
        </template>
      </v-list-item>
      <v-list-item link to="/tasks/waiting">
        <v-list-item-title>Waiting</v-list-item-title>
        <template v-slot:append>
          <v-badge color="warning" :content="stats.waiting" inline></v-badge>
        </template>
      </v-list-item>
      <v-list-item link to="/tasks/delayed">
        <v-list-item-title>Delayed</v-list-item-title>
        <template v-slot:append>
          <v-badge color="warning" :content="stats.delayed" inline></v-badge>
        </template>
      </v-list-item>
    </v-list>
  </v-sheet>
</template>

<script lang="ts" setup>
import { QueueStats } from "./stats.interface";

defineProps<{
  stats: QueueStats;
}>();
</script>
