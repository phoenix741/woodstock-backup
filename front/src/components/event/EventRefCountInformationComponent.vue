<template>
  <v-chip class="ma-2" label><v-icon icon="mdi-dots-grid" start></v-icon>{{ toNumber(information.count) }}</v-chip>
  <v-chip class="ma-2" label><v-icon icon="mdi-space-invaders" start></v-icon>{{ toNumber(information.error) }}</v-chip>
</template>

<script setup lang="ts">
import { EventRefCountInformation } from '@/generated/graphql';
import { toNumber } from '../hosts/hosts.utils';

defineProps<{ information: EventRefCountInformation }>();
</script>
