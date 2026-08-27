<template>
  <v-card v-if="information.sharePath.length" variant="tonal" rounded="lg" style="max-width: 480px">
    <v-card-subtitle class="pb-0 pt-2">
      <v-icon icon="mdi-folder-outline" size="x-small" start></v-icon>
      Shares
    </v-card-subtitle>
    <v-list density="compact" class="bg-transparent pa-0">
      <v-list-item v-for="share in information.sharePath" :key="share" :title="share" />
    </v-list>
  </v-card>
</template>

<script setup lang="ts">
import type { FragmentType } from '@/generated';
import { useFragment } from '@/generated';
import { EventBackupInformationFragment } from './events.fragment';

const props = defineProps<{ information: FragmentType<typeof EventBackupInformationFragment> }>();

const information = useFragment(EventBackupInformationFragment, props.information);
</script>
