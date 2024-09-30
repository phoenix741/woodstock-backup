<template>
  <v-chip class="ma-2" label><v-icon icon="mdi-dots-grid" start></v-icon>{{ toNumber(information.inRefcnt) }}</v-chip>
  <v-chip class="ma-2" label><v-icon icon="mdi-delete" start></v-icon>{{ toNumber(information.inUnused) }}</v-chip>
  <v-chip class="ma-2" label
    ><v-icon icon="mdi-space-invaders" start></v-icon>{{ toNumber(information.inNothing) }}</v-chip
  >
  <v-chip class="ma-2" label><v-icon icon="mdi-balloon" start></v-icon>{{ toNumber(information.missing) }}</v-chip>
</template>

<script setup lang="ts">
import { EventPoolInformation } from '@/generated/graphql';
import { toNumber } from '../hosts/hosts.utils';

defineProps<{ information: EventPoolInformation }>();
</script>
