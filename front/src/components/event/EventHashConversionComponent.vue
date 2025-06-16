<template>
  <v-chip title="Chunk count" class="ma-2" label><v-icon icon="mdi-dots-grid" start></v-icon>{{
    toNumber(information.count) }}</v-chip>
  <v-chip title="Conversion algorithm" class="ma-2" label><v-icon icon="mdi-pound" start></v-icon>{{
    information.algorithm }}</v-chip>
</template>

<script setup lang="ts">
import type { FragmentType } from '@/generated';
import { useFragment } from '@/generated';
import { toNumber } from '../hosts/hosts.utils';
import { EventHashConversionInformationFragment } from './events.fragment';

const props = defineProps<{ information: FragmentType<typeof EventHashConversionInformationFragment> }>();

const information = useFragment(EventHashConversionInformationFragment, props.information);
</script>
