<template>
  <v-card class="pa-2" :loading="isDeviceFetching">
    <v-card-title class="d-flex justify-space-between align-center">
      <div class="d-flex align-center">
        <span class="text-overline">{{ device?.name }}</span>
        <v-chip class="ml-2" :color="availabilityColor" size="small" rounded>
          {{ device?.availibilityState }}
        </v-chip>
      </div>
      <v-chip class="ml-2" label size="small" color="grey"> Agent v{{ agentVersion }} </v-chip>
    </v-card-title>

    <v-card-text>
      <!-- IP Addresses -->
      <div class="mb-2">
        <div class="text-subtitle-2">IP Addresses:</div>
        <v-chip-group>
          <v-chip v-for="(address, i) in device?.addresses" :key="i" size="x-small" variant="outlined">
            {{ address }}
          </v-chip>
        </v-chip-group>
      </div>

      <!-- Backup Information -->
      <v-row>
        <v-col cols="4">
          <div class="text-caption text-medium-emphasis">Last backup status</div>
          <div>
            <v-chip :color="colorState" size="small">
              {{ stateText }}
            </v-chip>
          </div>
        </v-col>
        <v-col cols="4">
          <div class="text-caption text-medium-emphasis">Next backup</div>
          <div>{{ dateToNextBackup }}</div>
        </v-col>
        <v-col cols="4">
          <div class="text-caption text-medium-emphasis">Last backup</div>
          <div>{{ timeSinceLastBackup }}</div>
        </v-col>
      </v-row>

      <v-row>
        <v-col>
          <!-- Pre-backup Commands -->
          <div class="mb-2" v-if="device?.configuration?.operations?.preCommands?.length">
            <div class="text-subtitle-2">Pre-backup Commands:</div>
            <v-list dense>
              <v-list-item v-for="(cmd, i) in device.configuration.operations.preCommands" :key="i">
                <v-list-item-title>{{ cmd.command }}</v-list-item-title>
              </v-list-item>
            </v-list>
          </div>

          <!-- Shares to backup -->
          <div v-if="device?.configuration?.operations?.operation?.shares?.length">
            <div class="text-subtitle-2">Shares to backup:</div>
            <v-list dense>
              <v-list-item v-for="(share, i) in device.configuration.operations.operation.shares" :key="i">
                <v-list-item-title>{{ share.name }}</v-list-item-title>
              </v-list-item>
            </v-list>
          </div>

          <!-- Post-backup Commands -->
          <div class="mb-2" v-if="device?.configuration?.operations?.postCommands?.length">
            <div class="text-subtitle-2">Post-backup Commands:</div>
            <v-list dense>
              <v-list-item v-for="(cmd, i) in device.configuration.operations.postCommands" :key="i">
                <v-list-item-title>{{ cmd.command }}</v-list-item-title>
              </v-list-item>
            </v-list>
          </div>
        </v-col>
      </v-row>
    </v-card-text>
  </v-card>
</template>

<script lang="ts" setup>
import { defineProps, computed } from 'vue';
import { useDevice } from '@/utils/devices';
import { getAvailabilityColor, getState, getStateColor, getStateText, toDateTime, toDuration } from '../hosts.utils';

const props = defineProps<{
  deviceId: string;
}>();

const { device, isDeviceFetching } = useDevice(props.deviceId);

const availabilityColor = computed(() => getAvailabilityColor(device.value?.availibilityState));

const agentVersion = computed(
  () => device.value?.agentVersion ?? device?.value?.lastBackup?.agentVersion ?? 'Unknown Version',
);

const dateToNextBackup = computed(() => device.value?.dateToNextBackup && toDateTime(device.value?.dateToNextBackup));

const timeSinceLastBackup = computed(
  () => device.value?.timeSinceLastBackup && toDuration(device.value?.timeSinceLastBackup * 1000),
);

const state = computed(() => device.value && getState(device.value));

const stateText = computed(() => state.value && getStateText(state.value));

const colorState = computed(() => state.value && getStateColor(state.value));
</script>
