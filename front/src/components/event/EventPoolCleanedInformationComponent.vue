<template>
  <v-chip class="ma-2" label><v-icon icon="mdi-file-outline" start></v-icon>{{ toNumber(information.count) }}</v-chip>
  <v-chip class="ma-2" label><v-icon icon="mdi-weight" start></v-icon>{{ filesize(information.size) }}</v-chip>
</template>

<script setup lang="ts">
import { EventPoolCleanedInformation } from '@/generated/graphql';
import filesize from '@/utils/filesize';
import { toNumber } from '../hosts/hosts.utils';

defineProps<{ information: EventPoolCleanedInformation }>();
</script>
