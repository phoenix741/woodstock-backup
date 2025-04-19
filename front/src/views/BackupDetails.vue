<template>
  <v-container>
    <v-row>
      <v-col>
        <v-progress-circular
          v-if="isFetching"
          indeterminate
          color="primary"
          size="64"
          width="6"
          class="ma-2"
        ></v-progress-circular>

        <template v-if="backup">
          <BackupFilesComponent :deviceId="deviceId" :backupNumber="backup.number"></BackupFilesComponent>

          <!-- <BackupLogComponent :deviceId="deviceId" :backupNumber="backup.number"></BackupLogComponent> -->
        </template>
      </v-col>
    </v-row>
  </v-container>
</template>

<script lang="ts" setup>
import BackupFilesComponent from '@/components/backups/BackupFilesComponent.vue';
// import BackupLogComponent from '@/components/backups/BackupLogComponent.vue';
import { useBackup } from '@/utils/backups';
import { useRoute } from 'vue-router';

const route = useRoute();

const deviceId = Array.isArray(route.params.deviceId) ? route.params.deviceId[0] : route.params.deviceId;
const backupId = parseInt(Array.isArray(route.params.backupId) ? route.params.backupId[0] : route.params.backupId);

const { backup, isFetching } = useBackup(deviceId, backupId);
</script>
