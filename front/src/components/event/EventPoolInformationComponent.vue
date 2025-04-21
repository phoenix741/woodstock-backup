<template>
  <v-chip title="In reference count" class="ma-2" label><v-icon icon="mdi-dots-grid" start></v-icon>{{
    toNumber(information.inRefcnt) }}</v-chip>
  <v-chip title="In unused (to clean)" class="ma-2" label><v-icon icon="mdi-delete" start></v-icon>{{
    toNumber(information.inUnused) }}</v-chip>

  <v-chip title="Number of references" class="ma-2" label><v-icon icon="mdi-counter" start></v-icon>{{
    toNumber(information.refcount) }}</v-chip>

  <v-chip title="Number of chunks" class="ma-2" label><v-icon icon="mdi-harddisk" start></v-icon>{{
    toNumber(information.chunkCount) }}</v-chip>

  <v-divider></v-divider>

  <v-chip title="Chunk neither in reference count nor in unused" class="ma-2" label><v-icon icon="mdi-space-invaders"
      start></v-icon>{{ toNumber(information.inNothing) }}</v-chip>
  <v-chip title="Missing chunks" class="ma-2" label><v-icon icon="mdi-balloon" start></v-icon>{{
    toNumber(information.missing) }}</v-chip>
  <v-chip title="Number of references in error" class="ma-2" label><v-icon icon="mdi-numeric-off" start></v-icon>{{
    toNumber(information.refcountError) }}</v-chip>
  <v-chip title="Number of chunks with the wrong hash" class="ma-2" label><v-icon icon="mdi-harddisk-remove"
      start></v-icon>{{
        toNumber(information.chunkError) }}</v-chip>

</template>

<script setup lang="ts">
import type { FragmentType } from '@/generated';
import { useFragment } from '@/generated';
import { toNumber } from '../hosts/hosts.utils';
import { EventPoolInformationFragment } from './events.fragment';

const props = defineProps<{ information: FragmentType<typeof EventPoolInformationFragment> }>();

const information = useFragment(EventPoolInformationFragment, props.information);
</script>
