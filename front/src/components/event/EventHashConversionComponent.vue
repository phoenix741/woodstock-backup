<template>
  <v-card variant="tonal" rounded="lg" style="max-width: 480px">
    <v-table density="compact" class="bg-transparent">
      <tbody>
        <tr>
          <td>Chunk count</td>
          <td class="text-right">{{ toNumber(information.count) }}</td>
        </tr>
        <tr>
          <td>Conversion algorithm</td>
          <td class="text-right">{{ information.algorithm }}</td>
        </tr>
      </tbody>
    </v-table>
  </v-card>
</template>

<script setup lang="ts">
import type { FragmentType } from '@/generated';
import { useFragment } from '@/generated';
import { toNumber } from '../hosts/hosts.utils';
import { EventHashConversionInformationFragment } from './events.fragment';

const props = defineProps<{ information: FragmentType<typeof EventHashConversionInformationFragment> }>();

const information = useFragment(EventHashConversionInformationFragment, props.information);
</script>
