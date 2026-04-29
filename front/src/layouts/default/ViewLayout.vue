<template>
  <v-main>
    <v-container fluid>
      <PoolHealthAlert />
      <router-view />
    </v-container>
  </v-main>
</template>

<script lang="ts" setup>
import PoolHealthAlert from '@/components/pool/PoolHealthAlert.vue';
</script>
