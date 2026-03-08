<template>
  <v-card class="pa-2" :loading="isFetching">
    <v-card-title class="d-flex justify-space-between align-center">
      <div>
        <span>Backup #{{ toNumber(backup?.number) }}</span>

        <v-chip class="ml-2" :color="statusColor" size="small">
          {{ getBackupStatusText(backup?.status) }}
        </v-chip>
      </div>
    </v-card-title>

    <v-card-text>
      <!-- Dates and duration -->
      <v-row>
        <v-col cols="3">
          <div class="text-caption text-medium-emphasis">Start</div>
          <div>{{ backup?.startDate && toDateTime(backup?.startDate) }}</div>
        </v-col>
        <v-col cols="3">
          <div class="text-caption text-medium-emphasis">End</div>
          <div>{{ backup?.endDate && toDateTime(backup?.endDate) }}</div>
        </v-col>
        <v-col cols="3">
          <div class="text-caption text-medium-emphasis">Duration</div>
          <div>{{ duration }}</div>
        </v-col>
        <v-col cols="3">
          <div class="text-caption text-medium-emphasis">Speed</div>
          <div>{{ filesize(backup?.speed ?? 0) }}/s</div>
        </v-col>
      </v-row>

      <!-- Errors -->
      <v-row>
        <v-col>
          <v-alert v-if="backup?.errorCount && backup.errorCount > 0" type="error" density="compact" variant="tonal"
            class="mb-2">
            {{ toNumber(backup.errorCount) }} error(s) detected
          </v-alert>
        </v-col>
      </v-row>

      <!-- File statistics -->
      <v-row>
        <v-col>
          <div class="text-subtitle-2 mb-1">File statistics</div>
          <v-table density="compact" class="bg-transparent">
            <tbody>
              <tr>
                <td>Total files</td>
                <td class="text-right">{{ toNumber(backup?.fileCount) }}</td>
              </tr>
              <tr>
                <td>New files</td>
                <td class="text-right">{{ toNumber(backup?.newFileCount) }}</td>
              </tr>
              <tr>
                <td>Existing files</td>
                <td class="text-right">{{ toNumber(backup?.existingFileCount) }}</td>
              </tr>
              <tr>
                <td>Deleted files</td>
                <td class="text-right">{{ toNumber(backup?.removedFileCount) }}</td>
              </tr>
              <tr>
                <td>Modified files</td>
                <td class="text-right">{{ toNumber(backup?.modifiedFileCount) }}</td>
              </tr>
            </tbody>
          </v-table>
        </v-col>

        <v-col>
          <div class="text-subtitle-2 mb-1">Sizes</div>
          <v-table density="compact" class="bg-transparent">
            <tbody>
              <tr v-if="backup?.fileSize">
                <td>Total size</td>
                <td class="text-right">{{ filesize(backup?.fileSize) }}</td>
              </tr>
              <tr v-if="backup?.newFileSize">
                <td>New files</td>
                <td class="text-right">{{ filesize(backup?.newFileSize) }}</td>
              </tr>
              <tr v-if="backup?.existingFileSize">
                <td>Existing files</td>
                <td class="text-right">{{ filesize(backup?.existingFileSize) }}</td>
              </tr>
            </tbody>
          </v-table>
        </v-col>
      </v-row>
    </v-card-text>
  </v-card>
</template>

<script lang="ts" setup>
import { defineProps, computed } from 'vue';
import { getBackupStatusColor, getBackupStatusText, useBackup } from '@/utils/backups';
import filesize from '@/utils/filesize';
import { parseDateTime, toDateTime, toDuration, toNumber } from '@/components/hosts/hosts.utils';

const props = defineProps<{
  deviceId: string;
  backupId: string;
}>();

const { backup, isFetching } = useBackup(props.deviceId, props.backupId);

const duration = computed(() => {
  if (backup?.value?.endDate) {
    return toDuration(parseDateTime(backup.value.endDate).getTime() - parseDateTime(backup.value.startDate).getTime());
  }
  return undefined;
});

const statusColor = computed(() => getBackupStatusColor(backup?.value?.status));
</script>
