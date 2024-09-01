<template>
  <v-container>
    <v-row v-if="backups && (backups?.length ?? 0) >= 3">
      <v-col>
        <BackupsChartsComponent :backups="backups"></BackupsChartsComponent>
      </v-col>
    </v-row>
    <v-row>
      <v-col>
        <v-sheet rounded="lg">
          <v-data-table
            show-select
            v-model="selection"
            v-model:items-per-page="itemsPerPage"
            :headers="headers"
            :items="backupsDataTable"
            :loading="isFetching"
            :sort-by="[{ key: 'number', order: 'desc' }]"
            loading-title="Loading... Please wait"
            item-value="number"
            class="elevation-1"
            @click:row="
              (_event: unknown, { item }: any) => {
                showDialog[item.number] = true;
              }
            "
          >
            <template v-slot:[`item.number`]="{ item }">
              <BackupView v-model="showDialog[item.number]" :deviceId="deviceId" :backup="item"></BackupView
              >{{ item.number }}
            </template>
            <template v-slot:[`item.startDate`]="{ item }">{{ toDateTime(item.startDate * 1000) }}</template>
            <template v-slot:[`item.fileSize`]="{ item }">{{ filesize(item.fileSize) }}</template>
            <template v-slot:[`item.existingFileSize`]="{ item }">{{ filesize(item.existingFileSize) }}</template>
            <template v-slot:[`item.newFileSize`]="{ item }">{{ filesize(item.newFileSize) }}</template>
            <template v-slot:[`item.fileCount`]="{ item }">{{ toNumber(item.fileCount) }}</template>
            <template v-slot:[`item.existingFileCount`]="{ item }">{{ toNumber(item.existingFileCount) }}</template>
            <template v-slot:[`item.newFileCount`]="{ item }">{{ toNumber(item.newFileCount) }}</template>
            <template v-slot:[`item.removedFileCount`]="{ item }">{{ toNumber(item.removedFileCount) }}</template>
            <template v-slot:[`item.modifiedFileCount`]="{ item }">{{ toNumber(item.modifiedFileCount) }}</template>

            <template v-slot:[`item.errorCount`]="{ item }">
              <v-chip :color="toErrorCountColor(item.errorCount)" size="small" label :text="toNumber(item.errorCount)">
              </v-chip>
            </template>

            <template v-slot:[`item.completed`]="{ item }">
              <v-progress-circular
                v-if="!item.endDate"
                indeterminate
                color="primary"
                width="20"
                size="20"
              ></v-progress-circular>

              <v-checkbox v-if="item.endDate" readonly v-model="item.completed" disabled></v-checkbox>
            </template>
            <template v-slot:bottom>
              <div class="d-flex">
                <div class="flex-1-0 text-left">
                  <BackupDownload :device-id="deviceId"></BackupDownload>
                </div>
                <div class="text-right pa-2">
                  <BackupDelete :device-id="deviceId" :backup-numbers="selection ?? []"></BackupDelete>
                  <BackupCreate :device-id="deviceId"></BackupCreate>
                </div>
              </div>
            </template>
          </v-data-table>
        </v-sheet>
      </v-col>
    </v-row>
  </v-container>
</template>

<script lang="ts" setup>
import BackupsChartsComponent from '@/components/backups/cards/BackupsChartsComponent.vue';
import { toDateTime, toMinutes, toNumber } from '@/components/hosts/hosts.utils';
import filesize from '@/utils/filesize';
import { computed, ref, watch } from 'vue';
import { useRoute } from 'vue-router';
import { VDataTable } from 'vuetify/components';

import { useBackups } from '../utils/backups';
import BackupCreate from './dialogs/BackupCreate.vue';
import BackupDownload from './dialogs/BackupDownload.vue';
import BackupDelete from './dialogs/BackupDelete.vue';
import BackupView from './dialogs/BackupView.vue';

type ReadonlyHeaders = VDataTable['$props']['headers'];

const route = useRoute();

const deviceId = Array.isArray(route.params.deviceId) ? route.params.deviceId[0] : route.params.deviceId;

let itemsPerPage = ref(25);

const headers: ReadonlyHeaders = [
  {
    title: 'Backup#',
    align: 'start',
    sortable: true,
    key: 'number',
  },
  { title: 'Start date', align: 'end', key: 'startDate' },
  { title: 'Duration (minutes)', align: 'end', key: 'duration' },

  { title: 'Files Count', align: 'end', key: 'fileCount' },
  { title: 'Files Size', align: 'end', key: 'fileSize' },

  { title: 'Existing Files Count', align: 'end', key: 'existingFileCount' },
  { title: 'Existing Files Size', align: 'end', key: 'existingFileSize' },

  { title: 'New Files Count', align: 'end', key: 'newFileCount' },
  { title: 'New Files Size', align: 'end', key: 'newFileSize' },

  { title: 'Modified Files Count', align: 'end', key: 'modifiedFileCount' },
  { title: 'Removed Files Count', align: 'end', key: 'removedFileCount' },

  { title: 'Error Count', align: 'end', key: 'errorCount' },

  { title: 'Complete', align: 'center', key: 'completed' },
];

const { backups, isFetching } = useBackups(deviceId);

const backupsDataTable = computed(() => {
  return backups.value?.map((backup) => ({
    duration: backup.endDate && toMinutes((backup.endDate - backup.startDate) * 1000),
    ...backup,
  }));
});

const selection = ref<number[]>([]);
const showDialog = ref<Record<number, boolean>>({});

watch(
  () => backups.value,
  () => {
    showDialog.value = backups.value?.reduce((acc, backup) => ({ ...acc, [backup.number]: false }), {}) ?? {};
  },
);

function toErrorCountColor(errorCount: number): string {
  return errorCount > 0 ? 'error' : 'success';
}
</script>
