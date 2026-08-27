<template>
  <v-card variant="tonal" rounded="lg" style="max-width: 480px">
    <v-table density="compact" class="bg-transparent">
      <tbody>
        <tr>
          <td>File count</td>
          <td class="text-right">{{ toNumber(information.count) }}</td>
        </tr>
        <tr>
          <td>Total size</td>
          <td class="text-right">{{ filesize(information.size) }}</td>
        </tr>
        <tr v-if="information.removedHashes.length">
          <td>Removed chunk hashes</td>
          <td class="text-right">
            <v-tooltip location="bottom">
              <template #activator="{ props: tooltipProps }">
                <span v-bind="tooltipProps">{{ toNumber(information.removedHashes.length) }}</span>
              </template>
              <div v-for="hash in information.removedHashes" :key="hash">{{ hash }}</div>
            </v-tooltip>
          </td>
        </tr>
      </tbody>
    </v-table>
  </v-card>
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
