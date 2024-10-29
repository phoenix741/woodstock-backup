<template>
  <v-chip class="ma-2" label><v-icon icon="mdi-dots-grid" start></v-icon>{{ toNumber(information.count) }}</v-chip>
  <v-chip class="ma-2" label><v-icon icon="mdi-space-invaders" start></v-icon>{{ toNumber(information.error) }}</v-chip>
</template>

<script setup lang="ts">
import { FragmentType, useFragment } from '@/generated';
import { toNumber } from '../hosts/hosts.utils';
import { EventRefCountInformationFragment } from './events.fragment';

const props = defineProps<{ information: FragmentType<typeof EventRefCountInformationFragment> }>();

const information = useFragment(EventRefCountInformationFragment, props.information)
</script>
