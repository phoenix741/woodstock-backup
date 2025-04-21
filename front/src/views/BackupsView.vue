<template>
  <v-container>
    <v-row>
      <v-col>
        <HostCard :deviceId="deviceId"></HostCard>
      </v-col>
    </v-row>
    <v-row v-if="backups && (backups?.length ?? 0) >= 3">
      <v-col>
        <BackupsChartsComponent :backups="backups"></BackupsChartsComponent>
      </v-col>
    </v-row>
    <v-row>
      <v-col>
        <v-sheet rounded="lg">
          <v-data-table show-select v-model="selection" v-model:items-per-page="itemsPerPage" :headers="headers"
            :items="backupsDataTable" :loading="isFetching" :sort-by="[{ key: 'number', order: 'desc' }]"
            loading-title="Loading... Please wait" item-value="number" class="elevation-1" @click:row="navigateTo">
            <template v-slot:[`item.number`]="{ item }">{{ item.number }}</template>
            <template v-slot:[`item.startDate`]="{ item }">{{ toDateTime(item.startDate * 1000) }}</template>
            <template v-slot:[`item.fileCount`]="{ item }">{{ toNumber(item.fileCount) }} ({{ filesize(item.fileSize)
              }})</template>
            <template v-slot:[`item.newFileCount`]="{ item }">{{ toNumber(item.newFileCount) }} ({{
              filesize(item.newFileSize) }})</template>
            <template v-slot:[`item.removedFileCount`]="{ item }">{{ toNumber(item.removedFileCount) }}</template>
            <template v-slot:[`item.modifiedFileCount`]="{ item }">{{ toNumber(item.modifiedFileCount) }}</template>

            <template v-slot:[`item.errorCount`]="{ item }">
              <v-chip :color="toErrorCountColor(item.errorCount)" size="small" label :text="toNumber(item.errorCount)">
              </v-chip>
            </template>

            <template v-slot:[`item.completed`]="{ item }">
              <v-progress-circular v-if="!item.endDate" indeterminate color="primary" width="20"
                size="20"></v-progress-circular>

              <v-chip :color="item.statusColor" size="small" label :text="item.status"> </v-chip>
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
import HostCard from '@/components/hosts/cards/HostCard.vue';
import { toDateTime, toDuration, toNumber } from '@/components/hosts/hosts.utils';
import filesize from '@/utils/filesize';
import { computed, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { type VDataTable } from 'vuetify/components';

import { getBackupStatusColor, useBackups } from '../utils/backups';
import BackupCreate from './dialogs/BackupCreate.vue';
import BackupDownload from './dialogs/BackupDownload.vue';
import BackupDelete from './dialogs/BackupDelete.vue';

type ReadonlyHeaders = VDataTable['$props']['headers'];

const route = useRoute();
const router = useRouter();

const deviceId = Array.isArray(route.params.deviceId) ? route.params.deviceId[0] : route.params.deviceId;

const itemsPerPage = ref(25);

const headers: ReadonlyHeaders = [
  {
    title: 'Backup#',
    align: 'start',
    sortable: true,
    key: 'number',
  },
  { title: 'Start date', align: 'end', key: 'startDate' },
  { title: 'Duration (minutes)', align: 'end', key: 'duration' },

  { title: 'Total Files', align: 'end', key: 'fileCount' },

  { title: 'New Files', align: 'end', key: 'newFileCount' },

  { title: 'Modified Files', align: 'end', key: 'modifiedFileCount' },
  { title: 'Removed Files', align: 'end', key: 'removedFileCount' },

  { title: 'Errors', align: 'end', key: 'errorCount' },

  { title: 'Complete', align: 'center', key: 'completed' },
];

const { backups, isFetching } = useBackups(deviceId);

const backupsDataTable = computed(() => {
  return backups.value?.map((backup) => ({
    duration: backup.endDate && toDuration((backup.endDate - backup.startDate) * 1000),
    statusColor: getBackupStatusColor(backup.status),
    ...backup,
  }));
});

const selection = ref<number[]>([]);

function toErrorCountColor(errorCount: number): string {
  return errorCount > 0 ? 'error' : 'success';
}

function navigateTo(event: PointerEvent, { item }: { item: Record<string, unknown> }) {
  router.push(`/backups/${deviceId}/${item.number}`);
}
</script>
