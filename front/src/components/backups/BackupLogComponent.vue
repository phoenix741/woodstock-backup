<template>
  <v-row>
    <v-col cols="2">
      <v-list>
        <v-list-item
          @click="logComponent = `/api/hosts/${deviceId}/backups/${backupNumber}/log/backup.log?tailable=false`"
        >
          <v-list-item-title>Transfert Logs</v-list-item-title>
        </v-list-item>
        <v-list-item
          @click="logComponent = `/api/hosts/${deviceId}/backups/${backupNumber}/log/backup.error.log?tailable=false`"
        >
          <v-list-item-title>Error Logs</v-list-item-title>
        </v-list-item>
        <v-list-item
          v-for="share in shares"
          :key="share.path"
          @click="logComponent = `/api/hosts/${deviceId}/backups/${backupNumber}/xferLog/${share.path}.log`"
        >
          <v-list-item-title>{{ unmangle(share.path) }}</v-list-item-title>
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
import { useShare } from '../../utils/backups';
import { unmangle } from '../../utils/file';

const props = defineProps<{
  deviceId: string;
  backupNumber: number;
}>();

const { shares } = useShare(props.deviceId, props.backupNumber);

const logComponent = ref(`/api/hosts/${props.deviceId}/backups/${props.backupNumber}/log/backup.log?tailable=false`);
</script>
