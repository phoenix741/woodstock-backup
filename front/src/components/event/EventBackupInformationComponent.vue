<template>
  <v-chip title="Share" v-for="share of information.sharePath" :key="share" class="ma-1" size="small" label
    ><v-icon icon="mdi-share-variant" start></v-icon>{{ share }}</v-chip
  >
</template>

<script setup lang="ts">
import type { FragmentType } from '@/generated';
import { useFragment } from '@/generated';
import { EventBackupInformationFragment } from './events.fragment';

const props = defineProps<{ information: FragmentType<typeof EventBackupInformationFragment> }>();

const information = useFragment(EventBackupInformationFragment, props.information);
</script>
