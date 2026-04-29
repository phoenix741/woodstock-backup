<template>
  <v-app>
    <Menu></Menu>
    <default-view />
  </v-app>
</template>

<script lang="ts" setup>
import Menu from '@/components/MenuComponent.vue';
import DefaultView from './ViewLayout.vue';
</script>
