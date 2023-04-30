<template>
  <v-dialog v-model="dialog" transition="dialog-bottom-transition">
    <v-card height="700px">
      <v-toolbar dark color="background">
        <v-toolbar-title>Backup {{ backup.number }}</v-toolbar-title>
        <template v-slot:extension>
          <v-tabs v-model="tab" align-tabs="title">
            <v-tab value="informations">Informations</v-tab>
            <v-tab value="files">Files</v-tab>
            <v-tab value="logs">Logs</v-tab>
          </v-tabs>
        </template>
      </v-toolbar>

      <v-card-text class="justify-center">
        <v-window v-model="tab">
          <v-window-item value="informations">
            <BackupInformationComponent :backup="backup"></BackupInformationComponent>
          </v-window-item>

          <v-window-item value="files">
            <BackupFilesComponent :deviceId="deviceId" :backupNumber="backup.number"></BackupFilesComponent>
          </v-window-item>

          <v-window-item value="logs">
            <BackupLogComponent :deviceId="deviceId" :backupNumber="backup.number"></BackupLogComponent>
          </v-window-item>
        </v-window>
      </v-card-text>
      <v-card-actions class="justify-center">
        <v-btn color="primary" block variant="text" @click="dialog = false"> Close </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script lang="ts" setup>
import BackupFilesComponent from '@/components/backups/BackupFilesComponent.vue';
import BackupInformationComponent from '@/components/backups/BackupInformationComponent.vue';
import BackupLogComponent from '@/components/backups/BackupLogComponent.vue';
import { BackupsQuery } from '@/generated/graphql';
import { computed, ref } from 'vue';

const props = defineProps<{
  modelValue: boolean;
  deviceId: string;
  backup: BackupsQuery['backups'][0];
}>();

const emit = defineEmits(['update:modelValue']);

const dialog = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

const tab = ref('informations');
</script>
