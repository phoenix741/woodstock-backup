<script setup lang="ts">
defineProps<{
  color: string;
  icon: string;
  label: string;
  value: number;
}>();
</script>

<template>
  <v-card class="mx-auto ma-5" :min-width="280" :max-width="280">
    <v-card-header>
      <v-card-header-text>
        <div class="text-overline">{{ label }}</div>
        <v-list-item-title class="headline">{{ value }}</v-list-item-title>
      </v-card-header-text>
      <v-card-avatar>
        <v-avatar :color="color">
          <v-icon color="white">{{ icon }}</v-icon>
        </v-avatar>
      </v-card-avatar>
    </v-card-header>

    <v-divider></v-divider>
    <v-card-text>
      <v-icon class="mr-2" small>mdi-clock-outline</v-icon>
      <span class="text-caption text-grey font-weight-light">
        <slot></slot>
      </span>
    </v-card-text>
  </v-card>
</template>
