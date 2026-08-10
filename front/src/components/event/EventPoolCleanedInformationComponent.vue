<template>
  <v-chip title="File count" class="ma-2" label
    ><v-icon icon="mdi-file-outline" start></v-icon>{{ toNumber(information.count) }}</v-chip
  >
  <v-chip title="Total size" class="ma-2" label
    ><v-icon icon="mdi-weight" start></v-icon>{{ filesize(information.size) }}</v-chip
  >
  <v-tooltip v-if="information.removedHashes.length" location="bottom">
    <template #activator="{ props: tooltipProps }">
      <v-chip v-bind="tooltipProps" title="Removed chunk hashes" class="ma-2" label
        ><v-icon icon="mdi-identifier" start></v-icon
        >{{ toNumber(information.removedHashes.length) }}</v-chip
      >
    </template>
    <div v-for="hash in information.removedHashes" :key="hash">{{ hash }}</div>
  </v-tooltip>
</template>

<script setup lang="ts">
import type { FragmentType } from '@/generated';
import { useFragment } from '@/generated';
import filesize from '@/utils/filesize';
import { toNumber } from '../hosts/hosts.utils';
import { EventPoolCleanedInformationFragment } from './events.fragment';

const props = defineProps<{ information: FragmentType<typeof EventPoolCleanedInformationFragment> }>();

const information = useFragment(EventPoolCleanedInformationFragment, props.information);
</script>
