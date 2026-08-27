<template>
  <v-row dense>
    <v-col cols="12" sm="6">
      <v-card variant="tonal" rounded="lg">
        <v-card-subtitle class="pb-0 pt-2">
          <v-icon icon="mdi-database-check-outline" size="x-small" start></v-icon>
          Content
        </v-card-subtitle>
        <v-table density="compact" class="bg-transparent">
          <tbody>
            <tr>
              <td>In reference count</td>
              <td class="text-right">{{ toNumber(information.inRefcnt) }}</td>
            </tr>
            <tr>
              <td>In unused (to clean)</td>
              <td class="text-right">{{ toNumber(information.inUnused) }}</td>
            </tr>
            <tr>
              <td>Number of references</td>
              <td class="text-right">{{ toNumber(information.refcount) }}</td>
            </tr>
            <tr>
              <td>Number of chunks</td>
              <td class="text-right">{{ toNumber(information.chunkCount) }}</td>
            </tr>
          </tbody>
        </v-table>
      </v-card>
    </v-col>

    <v-col v-if="errorCount > 0" cols="12" sm="6">
      <v-card variant="tonal" color="error" rounded="lg">
        <v-card-subtitle class="pb-0 pt-2 text-error">
          <v-icon icon="mdi-alert-outline" size="x-small" start></v-icon>
          Errors
        </v-card-subtitle>
        <v-table density="compact" class="bg-transparent">
          <tbody>
            <tr v-if="information.inNothing">
              <td>Orphan chunks (in neither refcount nor unused)</td>
              <td class="text-right">{{ toNumber(information.inNothing) }}</td>
            </tr>
            <tr v-if="information.missing">
              <td>Missing chunks</td>
              <td class="text-right">{{ toNumber(information.missing) }}</td>
            </tr>
            <tr v-if="information.refcountError">
              <td>References in error</td>
              <td class="text-right">{{ toNumber(information.refcountError) }}</td>
            </tr>
            <tr v-if="information.chunkError">
              <td>Chunks with the wrong hash</td>
              <td class="text-right">{{ toNumber(information.chunkError) }}</td>
            </tr>
          </tbody>
        </v-table>
      </v-card>
    </v-col>
  </v-row>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { FragmentType } from '@/generated';
import { useFragment } from '@/generated';
import { toNumber } from '../hosts/hosts.utils';
import { EventPoolInformationFragment } from './events.fragment';

const props = defineProps<{ information: FragmentType<typeof EventPoolInformationFragment> }>();

const information = useFragment(EventPoolInformationFragment, props.information);

const errorCount = computed(
  () => information.inNothing + information.missing + information.refcountError + information.chunkError,
);
</script>
