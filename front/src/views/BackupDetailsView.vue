<template>
  <v-container>
    <v-row>
      <v-col>
        <BackupCard :deviceId="deviceId" :backupId="backupId"></BackupCard>
      </v-col>
    </v-row>
    <v-row>
      <v-col>
        <v-card class="mx-auto">
          <v-card-title class="text-overline">Files</v-card-title>
          <v-card-text>
            <BackupFilesComponent :deviceId="deviceId" :backupNumber="backupId"></BackupFilesComponent>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
    <v-row>
      <v-col>
        <v-card class="mx-auto">
          <v-card-title class="text-overline">Logs</v-card-title>
          <v-card-text>
            <BackupLogComponent :deviceId="deviceId" :backupNumber="backupId"></BackupLogComponent>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script lang="ts" setup>
import BackupFilesComponent from '@/components/backups/BackupFilesComponent.vue';
import BackupCard from '@/components/backups/cards/BackupCard.vue';
import BackupLogComponent from '@/components/backups/BackupLogComponent.vue';
import { useRoute } from 'vue-router';

const route = useRoute();

const deviceId = Array.isArray(route.params.deviceId) ? route.params.deviceId[0] : route.params.deviceId;
const backupId = parseInt(Array.isArray(route.params.backupId) ? route.params.backupId[0] : route.params.backupId);
</script>
