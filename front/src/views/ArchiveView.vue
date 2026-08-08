<template>
  <v-container>
    <v-row>
      <v-col>
        <v-sheet rounded="lg">
          <v-data-table
            v-model:items-per-page="itemsPerPage"
            :headers="headers"
            :items="profilesDataTable"
            :loading="isArchiveProfilesFetching"
            loading-text="Loading... Please wait"
            item-value="name"
            class="elevation-1"
            @click:row="navigateTo"
          >
            <template v-slot:[`item.enabled`]="{ item }">
              <v-chip :color="item.enabled ? 'success' : 'default'" size="small" variant="tonal">
                {{ item.enabled ? 'enabled' : 'disabled' }}
              </v-chip>
            </template>
            <template v-slot:[`item.format`]="{ item }">
              <v-chip color="blue" size="small" variant="tonal">{{ item.formatLabel }}</v-chip>
            </template>
          </v-data-table>
        </v-sheet>
      </v-col>
    </v-row>

    <v-row v-if="!isArchiveProfilesFetching && profiles.length === 0" class="mt-10" justify="center">
      <v-col cols="auto" class="text-center">
        <v-icon size="72" color="grey-lighten-1">mdi-archive-outline</v-icon>
        <div class="text-body-1 text-grey mt-2">No archive profile configured</div>
        <div class="text-body-2 text-grey">Profiles are defined in <code>archiving.yml</code>.</div>
      </v-col>
    </v-row>
  </v-container>
</template>

<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useRouter } from 'vue-router';
import { type VDataTable } from 'vuetify/components';

import { archiveFormatLabel, hostSelectionSummary, useArchiveProfiles } from '@/utils/archiving';

type ReadonlyHeaders = VDataTable['$props']['headers'];

const router = useRouter();

const { profiles, isArchiveProfilesFetching } = useArchiveProfiles();

const itemsPerPage = ref(25);

const headers: ReadonlyHeaders = [
  { title: 'Profile', align: 'start', sortable: true, key: 'name' },
  { title: 'Format', align: 'start', key: 'format' },
  { title: 'Destination', align: 'start', key: 'destination' },
  { title: 'Host selection', align: 'start', key: 'hostSelection' },
  { title: 'Schedule (cron)', align: 'start', key: 'scheduleCron' },
  { title: 'State', align: 'end', key: 'enabled' },
];

const profilesDataTable = computed(() =>
  profiles.value.map((profile) => ({
    ...profile,
    formatLabel: archiveFormatLabel(profile.format),
    hostSelection: hostSelectionSummary(profile),
  })),
);

function navigateTo(event: PointerEvent, { item }: { item: Record<string, unknown> }) {
  router.push(`/archive/${item.name}`);
}
</script>
