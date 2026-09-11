<template>
  <v-main>
    <v-container fluid>
      <PoolHealthAlert v-if="isAdmin" />
      <router-view />
    </v-container>
  </v-main>
</template>

<script lang="ts" setup>
import { computed } from 'vue';
import PoolHealthAlert from '@/components/pool/PoolHealthAlert.vue';
import { useAuth } from '@/composables/useAuth';

const auth = useAuth();
// Pool health/statistics are admin-only server-side — not admin-gated when auth is
// disabled, same "unrestricted" default as the server.
const isAdmin = computed(() => !auth.enabled.value || auth.isAdmin.value);
</script>
