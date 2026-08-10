<template>
  <v-card class="pa-2" :loading="isArchiveProfilesFetching">
    <v-card-title class="d-flex justify-space-between align-center">
      <div class="d-flex align-center">
        <span class="text-overline">{{ profile?.name }}</span>
        <v-chip class="ml-2" :color="profile?.enabled ? 'success' : 'default'" size="small" rounded>
          {{ profile?.enabled ? 'enabled' : 'disabled' }}
        </v-chip>
      </div>
      <v-chip class="ml-2" label size="small" color="blue" v-if="profile">{{ formatLabel }}</v-chip>
    </v-card-title>

    <v-card-text v-if="profile">
      <v-row>
        <v-col cols="4">
          <div class="text-caption text-medium-emphasis">Destination</div>
          <div>{{ profile.destination }}</div>
        </v-col>
        <v-col cols="4">
          <div class="text-caption text-medium-emphasis">Schedule (cron)</div>
          <div>{{ profile.scheduleCron }}</div>
        </v-col>
        <v-col cols="4">
          <div class="text-caption text-medium-emphasis">Checksum file (.sha256)</div>
          <div>{{ profile.checksum ? 'Yes' : 'No' }}</div>
        </v-col>
        <v-col cols="4" v-if="isCompressedFormat(profile.format)">
          <div class="text-caption text-medium-emphasis">Compression level</div>
          <div>{{ profile.compressionLevel ?? 'default' }}</div>
        </v-col>
      </v-row>

      <div class="mt-2">
        <div class="text-subtitle-2">Host selection: {{ modeLabel }}</div>
        <v-chip-group v-if="profile.hostSelectionMode === HostSelectionMode.Include">
          <v-chip v-for="host in profile.hostSelectionHosts" :key="host" size="x-small" variant="outlined">
            {{ host }}
          </v-chip>
        </v-chip-group>
        <v-chip-group v-else-if="profile.hostSelectionMode === HostSelectionMode.Exclude">
          <v-chip v-for="host in profile.hostSelectionHosts" :key="host" size="x-small" variant="outlined">
            {{ host }}
          </v-chip>
        </v-chip-group>
        <div v-else-if="profile.hostSelectionMode === HostSelectionMode.Glob" class="text-body-2">
          <code>{{ profile.hostSelectionPattern }}</code>
        </div>
      </div>
    </v-card-text>
  </v-card>
</template>

<script lang="ts" setup>
import { computed } from 'vue';
import { HostSelectionMode } from '@/generated/graphql';
import { archiveFormatLabel, hostSelectionModeLabel, isCompressedFormat, useArchiveProfile } from '@/utils/archiving';

const props = defineProps<{
  profileName: string;
}>();

const { profile, isArchiveProfilesFetching } = useArchiveProfile(props.profileName);

const formatLabel = computed(() => profile.value && archiveFormatLabel(profile.value.format));
const modeLabel = computed(() => profile.value && hostSelectionModeLabel(profile.value.hostSelectionMode));
</script>
