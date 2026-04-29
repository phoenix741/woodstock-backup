<template>
  <v-container>
    <v-row>
      <v-col cols="3">
        <HostSuccessFailureChartsComponent v-if="!isDeviceFetching" :countByState="devicesByState">
        </HostSuccessFailureChartsComponent>
      </v-col>
      <v-col cols="9">
        <HostRepartitionChartsComponent v-if="!isStatsFetching" :hosts="devicesBySize"></HostRepartitionChartsComponent>
      </v-col>
    </v-row>
    <v-row>
      <v-col>
        <v-sheet rounded="lg">
          <v-data-table
            v-model:items-per-page="itemsPerPage"
            :headers="headers"
            :items="devicesDataTable"
            :loading="isDeviceFetching"
            loading-text="Loading... Please wait"
            item-value="name"
            item-title="name"
            class="elevation-1"
            @click:row="navigateTo"
          >
            <template v-slot:[`item.state`]="{ item }">
              <v-chip v-if="item.state" :color="getStateColor(item.state)">{{ getStateText(item.state) }}</v-chip>
            </template>
            <template v-slot:[`item.lastBackupAge`]="{ item }">
              <template v-if="item.lastBackupAge">{{ toDuration(item.lastBackupAge * 1000) }}</template>
            </template>
            <template v-slot:[`item.nextBackup`]="{ item }">
              <template v-if="item.nextBackup">{{ toDateTime(item.nextBackup) }}</template>
            </template>
            <template v-slot:[`item.lastBackupSize`]="{ item }">
              <template v-if="item.lastBackupSize">{{ filesize(item.lastBackupSize) }}</template>
            </template>
            <template v-slot:[`item.lastBackupNumber`]="{ item }">
              <template v-if="item.lastBackupNumber !== null && item.lastBackupNumber !== undefined">{{
                toNumber(item.lastBackupNumber)
              }}</template>
            </template>
            <template v-slot:[`item.agentVersion`]="{ item }">
              {{ item.agentVersion || 'unknown' }}
            </template>
            <template v-slot:[`item.availibility`]="{ item }">
              <v-chip :color="item.availibilityColor" rounded>{{
                item.availibilityState?.toLocaleLowerCase() || 'unknown'
              }}</v-chip>
            </template>
            <template v-slot:bottom>
              <div class="d-flex">
                <div class="flex-1-0 text-left"></div>
                <div class="text-right pa-2">
                  <v-btn class="ml-1" color="primary" variant="text" @click="clearCache()">Clear cache</v-btn>
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
import HostRepartitionChartsComponent from '@/components/hosts/cards/HostRepartitionChartsComponent.vue';
import HostSuccessFailureChartsComponent from '@/components/hosts/cards/HostSuccessFailureChartsComponent.vue';

import {
  getAvailabilityColor,
  getState,
  getStateColor,
  getStateText,
  toDateTime,
  toDuration,
  toNumber,
} from '@/components/hosts/hosts.utils';
import filesize from '@/utils/filesize';
import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import { type VDataTable } from 'vuetify/components';

import { useDevices } from '../utils/devices';
import { useDiskUsageStats } from '../utils/stats';

type ReadonlyHeaders = VDataTable['$props']['headers'];

const router = useRouter();
const { devices, isDeviceFetching, devicesByState, clearCache } = useDevices();
const { devicesBySize, isStatsFetching } = useDiskUsageStats();

function navigateTo(event: PointerEvent, { item }: { item: Record<string, unknown> }) {
  router.push(`/backups/${item.name}`);
}

const itemsPerPage = ref(25);

const headers: ReadonlyHeaders = [
  {
    title: 'Device',
    align: 'start',
    sortable: false,
    key: 'name',
  },
  { title: 'Last backup number', align: 'end', key: 'lastBackupNumber' },
  { title: 'Last backup age', align: 'end', key: 'lastBackupAge' },
  { title: 'Next Backup', align: 'end', key: 'nextBackup' },
  { title: 'Last backup size', align: 'end', key: 'lastBackupSize' },
  { title: 'Agent', align: 'start', key: 'agentVersion' },
  { title: 'Availability', align: 'start', key: 'availibility' },
  { title: 'State', align: 'end', key: 'state' },
];

const devicesDataTable = computed(() => {
  return devices.value?.hosts.map((device) => {
    return {
      name: device.name,
      lastBackupNumber: device.lastBackup?.number,
      lastBackupAge: device.timeSinceLastBackup,
      nextBackup: device.dateToNextBackup,
      lastBackupSize: device.lastBackup?.fileSize,
      state: getState(device),
      configuration: device.configuration,
      agentVersion: device.agentVersion || device.lastBackup?.agentVersion || '',
      availibilityState: device.availibilityState,
      availibilityColor: getAvailabilityColor(device.availibilityState),
    };
  });
});
</script>
