<template>
  <v-row>
    <v-col cols="6">
      <BackupFilesTreeComponent
        :deviceId="props.deviceId"
        :backupNumber="props.backupNumber"
        :hiddenFiles="false"
        @select="selected = $event"
      ></BackupFilesTreeComponent>
    </v-col>
    <v-col cols="6">
      <v-table v-if="selected">
        <tbody>
          <tr>
            <td>Path</td>
            <td>{{ selected.path.join('/') }}</td>
          </tr>
          <tr>
            <td>Type</td>
            <td>{{ selected.node.type }}</td>
          </tr>
          <tr v-if="selected.node.symlink">
            <td>Symlink</td>
            <td>{{ selected.node.symlink }}</td>
          </tr>
          <tr v-if="selected.node.stats?.ownerId">
            <td>Owner</td>
            <td>{{ selected.node.stats.ownerId }}</td>
          </tr>
          <tr v-if="selected.node.stats?.groupId">
            <td>Group</td>
            <td>{{ selected.node.stats.groupId }}</td>
          </tr>
          <tr v-if="selected.node.stats?.mode">
            <td>Mode</td>
            <td>{{ (selected.node.stats.mode & 0o7777).toString(8) }}</td>
          </tr>
          <tr v-if="selected.node.stats?.size">
            <td>Size</td>
            <td>{{ filesize(parseInt(selected.node.stats.size)) }}</td>
          </tr>
          <tr v-if="selected.node.stats?.lastModified">
            <td>Modification Time</td>
            <td>{{ toDateTime(parseInt(selected.node.stats.lastModified) * 1000) }}</td>
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
import { TreeViewNode } from './backups.interface';

const props = defineProps<{
  deviceId: string;
  backupNumber: number;
}>();

const selected = ref<TreeViewNode | undefined>(undefined);
const selectedPath = computed(
  () =>
    `/api/hosts/${props.deviceId}/backups/${props.backupNumber}/files/download?sharePath=${
      selected.value?.sharePath
    }&path=${selected.value?.path.join('/')}`,
);
</script>
