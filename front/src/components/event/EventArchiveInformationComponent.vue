<template>
  <v-card variant="tonal" rounded="lg" style="max-width: 480px">
    <v-table density="compact" class="bg-transparent">
      <tbody>
        <tr>
          <td>Profile</td>
          <td class="text-right">{{ information.profileName }}</td>
        </tr>
        <tr>
          <td>Hosts</td>
          <td class="text-right">{{ information.hostsDone }} / {{ information.hostsTotal }}</td>
        </tr>
        <tr v-if="information.failedHosts.length">
          <td>Failed hosts</td>
          <td class="text-right">
            <v-tooltip location="bottom">
              <template #activator="{ props: tooltipProps }">
                <span v-bind="tooltipProps">{{ toNumber(information.failedHosts.length) }}</span>
              </template>
              <div v-for="hostname in information.failedHosts" :key="hostname">{{ hostname }}</div>
            </v-tooltip>
          </td>
        </tr>
        <tr>
          <td>File count</td>
          <td class="text-right">{{ toNumber(information.fileCount) }}</td>
        </tr>
        <tr>
          <td>Archive size</td>
          <td class="text-right">{{ filesize(information.archiveSize) }}</td>
        </tr>
        <tr v-if="information.cancelled">
          <td>Cancelled</td>
          <td class="text-right">Yes</td>
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
import { EventArchiveInformationFragment } from './events.fragment';

const props = defineProps<{ information: FragmentType<typeof EventArchiveInformationFragment> }>();

const information = useFragment(EventArchiveInformationFragment, props.information);
</script>
