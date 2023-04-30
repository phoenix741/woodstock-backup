<template>
  <v-row>
    <v-col cols="2">
      <v-list>
        <v-list-item @click="logComponent = `/api/hosts/${deviceId}/backups/${backupNumber}/log/backup.log`">
          <v-list-item-title>Transfert Logs</v-list-item-title>
        </v-list-item>
        <v-list-item @click="logComponent = `/api/hosts/${deviceId}/backups/${backupNumber}/log/backup.error.log`">
          <v-list-item-title>Error Logs</v-list-item-title>
        </v-list-item>
      </v-list>
    </v-col>
    <v-col cols="10">
      <LogComponent :url="logComponent"></LogComponent>
      <a :href="logComponent" target="_blank">Show in a new tab</a>
    </v-col>
  </v-row>
</template>

<script lang="ts" setup>
import LogComponent from '@/components/backups/LogComponent.vue';
import { ref } from 'vue';

const props = defineProps<{
  deviceId: string;
  backupNumber: number;
}>();

const logComponent = ref(`/api/hosts/${props.deviceId}/backups/${props.backupNumber}/log/backup.log`);
</script>
