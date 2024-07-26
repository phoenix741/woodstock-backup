<template>
  <v-container>
    <v-row>
      <v-col cols="3">
        <HostSuccessFailureChartsComponent
          v-if="!isDeviceFetching"
          :countByState="devicesByState"
        ></HostSuccessFailureChartsComponent>
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
              <v-chip v-if="item.state" :color="getColor(item.state)">{{ item.state }}</v-chip>
            </template>
            <template v-slot:[`item.lastBackupSize`]="{ item }">
              <template v-if="item.lastBackupSize">{{ filesize(item.lastBackupSize) }}</template>
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

import { getColor, getState, toDay } from '@/components/hosts/hosts.utils';
import filesize from '@/utils/filesize';
import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import { VDataTable } from 'vuetify/components';

import { useDevices } from '../utils/devices';
import { useDiskUsageStats } from '../utils/stats';

type ReadonlyHeaders = VDataTable['$props']['headers'];

const router = useRouter();
const { devices, isDeviceFetching, devicesByState } = useDevices();
const { devicesBySize, isStatsFetching } = useDiskUsageStats();

function navigateTo(event: PointerEvent, { item }: { item: Record<string, unknown> }) {
  router.push(`/backups/${item.name}`);
}

let itemsPerPage = ref(25);

const headers: ReadonlyHeaders = [
  {
    title: 'Device',
    align: 'start',
    sortable: false,
    key: 'name',
  },
  { title: 'Last backup number', align: 'end', key: 'lastBackupNumber' },
  { title: 'Last backup age', align: 'end', key: 'lastBackupAge' },
  { title: 'Last backup size', align: 'end', key: 'lastBackupSize' },
  { title: 'State', align: 'end', key: 'state' },
];

const devicesDataTable = computed(() => {
  return devices.value?.hosts.map((device) => ({
    name: device.name,
    lastBackupNumber: device.lastBackup?.number,
    lastBackupAge:
      device.lastBackup && toDay(new Date().getTime() - new Date(device.lastBackup?.startDate * 1000).getTime()),
    lastBackupSize: device.lastBackup?.fileSize,
    state: getState(device),
    configuration: device.configuration,
  }));
});
</script>
