<template>
  <v-app>
    <Menu></Menu>
    <default-view />
    <v-footer app class="text-caption justify-center text-medium-emphasis">v{{ appVersion }}</v-footer>
  </v-app>
</template>

<script lang="ts" setup>
import Menu from '@/components/MenuComponent.vue';
import DefaultView from './ViewLayout.vue';

const appVersion = __APP_VERSION__;
</script>
