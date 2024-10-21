<template>
  <v-row>
    <v-col cols="6">
      <BackupFilesTreeComponent
        :deviceId="props.deviceId"
        :backupNumber="props.backupNumber"
        :hiddenFiles="false"
      ></BackupFilesTreeComponent>
    </v-col>
    <v-col cols="6" v-if="selected && selected?.file">
      <v-table>
        <tbody>
          <tr>
            <td>Path</td>
            <td>{{ selected.text }}</td>
          </tr>
          <tr>
            <td>Type</td>
            <td>{{ selected.file.type }}</td>
          </tr>
          <tr v-if="selected.file.symlink">
            <td>Symlink</td>
            <td>{{ selected.file.symlink }}</td>
          </tr>
          <tr v-if="selected.file.stats?.ownerId">
            <td>Owner</td>
            <td>{{ selected.file.stats.ownerId }}</td>
          </tr>
          <tr v-if="selected.file.stats?.groupId">
            <td>Group</td>
            <td>{{ selected.file.stats.groupId }}</td>
          </tr>
          <tr v-if="selected.file.stats?.mode">
            <td>Mode</td>
            <td>{{ (selected.file.stats.mode & 0o7777).toString(8) }}</td>
          </tr>
          <tr v-if="selected.file.stats?.size">
            <td>Size</td>
            <td>{{ filesize(parseInt(selected.file.stats.size)) }}</td>
          </tr>
          <tr v-if="selected.file.stats?.lastModified">
            <td>Modification Time</td>
            <td>{{ toDateTime(parseInt(selected.file.stats.lastModified) * 1000) }}</td>
          </tr>
        </tbody>
        <tfoot>
          <tr>
            <td colspan="2" class="text-right">
              <v-btn color="primary" :href="selectedPath" target="_blank">Download</v-btn>
            </td>
          </tr>
        </tfoot>
      </v-table>
    </v-col>
  </v-row>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { toDateTime } from '../hosts/hosts.utils';
import filesize from '@/utils/filesize';
import { computed } from 'vue';
import BackupFilesTreeComponent from '@/components/backups/BackupFilesTreeComponent.vue';

const props = defineProps<{
  deviceId: string;
  backupNumber: number;
}>();

const selected = ref<Node | undefined>(undefined);
const selectedPath = computed(
  () =>
    `/api/hosts/${props.deviceId}/backups/${props.backupNumber}/files/download?sharePath=${selected.value?.sharePath}&path=${selected.value?.file?.path}`,
);
</script>
