<template>
  <v-row>
    <v-col>
      <div class="font-weight-bold">Start date</div>
      <div>{{ toDateTime(backup.startDate * 1000) }}</div>
    </v-col>
    <v-col>
      <div class="font-weight-bold">End date</div>
      <div>{{ backup.endDate && toDateTime(backup.endDate * 1000) }}</div>
    </v-col>
    <v-col>
      <div class="font-weight-bold">Is backup complete</div>
      <div>
        <v-checkbox readonly :model-value="backup.completed" disabled></v-checkbox>
      </div>
    </v-col>
  </v-row>
  <v-row>
    <v-col>
      <div class="font-weight-bold">File count</div>
      <div>{{ toNumber(backup.fileCount) }}</div>
    </v-col>
    <v-col>
      <div class="font-weight-bold">New file count</div>
      <div>{{ toNumber(backup.newFileCount) }}</div>
    </v-col>
    <v-col>
      <div class="font-weight-bold">Existing file count</div>
      <div>{{ toNumber(backup.existingFileCount) }}</div>
    </v-col>
  </v-row>
  <v-row>
    <v-col>
      <v-progress-linear
        :model-value="backup.newFileCount"
        :max="backup.fileCount"
        height="24"
        bg-color="secondary"
        bg-opacity="1"
        color="primary"
      >
        <template v-slot:default="{}">
          <strong>{{ toNumber(backup.newFileCount) }} / {{ toNumber(backup.fileCount) }}</strong>
        </template>
      </v-progress-linear>
    </v-col>
  </v-row>
  <v-row>
    <v-col>
      <div class="font-weight-bold">File size</div>
      <div>{{ filesize(backup.fileSize) }}</div>
    </v-col>
    <v-col>
      <div class="font-weight-bold">New file size</div>
      <div>{{ filesize(backup.newFileSize) }}</div>
    </v-col>
    <v-col>
      <div class="font-weight-bold">Existing file size</div>
      <div>{{ filesize(backup.existingFileSize) }}</div>
    </v-col>
  </v-row>
  <v-row>
    <v-col>
      <v-progress-linear
        :model-value="Number(backup.newFileSize / 1024n / 1024n)"
        :max="Number(backup.fileSize / 1024n / 1024n)"
        height="24"
        bg-color="secondary"
        bg-opacity="1"
        color="primary"
      >
        <template v-slot:default="{}">
          <strong>{{ filesize(backup.newFileSize) }} / {{ filesize(backup.fileSize) }}</strong>
        </template></v-progress-linear
      >
    </v-col>
  </v-row>
  <v-row>
    <v-col>
      <div class="font-weight-bold">Speed</div>
      <div>{{ filesize(backup.speed) }}/s</div>
    </v-col>
  </v-row>
</template>

<script lang="ts" setup>
import { toDateTime, toNumber } from '@/components/hosts/hosts.utils';
import { BackupsQuery } from '@/generated/graphql';
import filesize from '@/utils/filesize';

defineProps<{
  backup: BackupsQuery['backups'][0];
}>();
</script>
