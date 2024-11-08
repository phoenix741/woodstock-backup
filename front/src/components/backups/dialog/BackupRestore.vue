<template>
  <v-btn class="ml-1" color="primary" variant="text"
    >Restore on device
    <v-dialog v-model="dialog" activator="parent" width="auto">
      <v-card rounded="lg">
        <v-card-title class="d-flex justify-space-between align-center">
          <div class="text-h5 text-medium-emphasis ps-2">Restore backup</div>

          <v-btn icon="mdi-close" variant="text" @click="dialog = false"></v-btn>
        </v-card-title>

        <v-divider class="mb-4"></v-divider>

        <v-card-text>
          <div class="text-medium-emphasis mb-4">
            Select the destination where the restauration of the path
            <code>{{ path }}</code>
            should be made.
          </div>

          <div class="text-subtitle-1 text-medium-emphasis">Destination directory</div>

          <v-text-field
            density="compact"
            :placeholder="destinationPath"
            v-model="destinationPath"
            prepend-inner-icon="mdi-folder"
            variant="outlined"
          ></v-text-field>
        </v-card-text>

        <v-divider class="mt-2"></v-divider>

        <v-card-actions class="my-2 d-flex justify-end">
          <v-btn class="text-none" rounded="xl" text="Cancel" @click="dialog = false"></v-btn>

          <v-btn class="text-none" color="primary" rounded="xl" text="Send" variant="flat" @click="restore()"></v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-btn>
</template>

<script lang="ts" setup>
import { ref } from 'vue';
import { useBackupRestore } from '@/utils/backups_restore';

const props = defineProps<{
  deviceId: string;
  backupNumber: number;
  sharePath: string;
  path: string;
}>();

const dialog = ref(false);
const destinationPath = ref('/');

const { restoreBackup } = useBackupRestore();

async function restore() {
  await restoreBackup(props.deviceId, props.backupNumber, props.sharePath, props.path, destinationPath.value);
  dialog.value = false;
}
</script>
