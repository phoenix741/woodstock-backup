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
            item-key="name"
            item-title="name"
            class="elevation-1"
            @click:row="
              (_event: unknown, { item }: any) => {
                showDialog[item.columns.number] = true;
              }
            "
          >
            <template v-slot:[`item.number`]="{ item }">
              <BackupView v-model="showDialog[item.columns.number]" :deviceId="deviceId" :backup="item.raw"></BackupView
              >{{ item.columns.number }}
            </template>
            <template v-slot:[`item.startDate`]="{ item }">{{ toDateTime(item.columns.startDate * 1000) }}</template>
            <template v-slot:[`item.fileSize`]="{ item }">{{ filesize(item.columns.fileSize) }}</template>
            <template v-slot:[`item.existingFileSize`]="{ item }">{{
              filesize(item.columns.existingFileSize)
            }}</template>
            <template v-slot:[`item.newFileSize`]="{ item }">{{ filesize(item.columns.newFileSize) }}</template>
            <template v-slot:[`item.fileCount`]="{ item }">{{ toNumber(item.columns.fileCount) }}</template>
            <template v-slot:[`item.existingFileCount`]="{ item }">{{
              toNumber(item.columns.existingFileCount)
            }}</template>
            <template v-slot:[`item.newFileCount`]="{ item }">{{ toNumber(item.columns.newFileCount) }}</template>
            <template v-slot:[`item.completed`]="{ item }">
              <v-checkbox readonly v-model="item.columns.completed" disabled></v-checkbox>
            </template>
            <template v-slot:bottom>
              <div class="text-right pa-2">
                <BackupDelete
                  :device-id="deviceId"
                  :backup-numbers="selection?.map((item) => item.number) || []"
                ></BackupDelete>
                <BackupCreate :device-id="deviceId"></BackupCreate>
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
import { useBackups } from '../utils/backups';
import BackupCreate from './dialogs/BackupCreate.vue';
import BackupDelete from './dialogs/BackupDelete.vue';
import BackupView from './dialogs/BackupView.vue';

const route = useRoute();

const deviceId = Array.isArray(route.params.deviceId) ? route.params.deviceId[0] : route.params.deviceId;

let itemsPerPage = ref(25);

const headers = [
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
  { title: 'Complete', align: 'center', key: 'completed' },
];

const { backups, isFetching } = useBackups(deviceId);

const backupsDataTable = computed(() => {
  return backups.value?.map((backup) => ({
    duration: backup.endDate && toMinutes((backup.endDate - backup.startDate) * 1000),
    ...backup,
  }));
});

const selection = ref<typeof backupsDataTable.value>([]);
const showDialog = ref<Record<number, boolean>>({});

watch(
  () => backups.value,
  () => {
    showDialog.value = backups.value?.reduce((acc, backup) => ({ ...acc, [backup.number]: false }), {}) ?? {};
  },
);
</script>
