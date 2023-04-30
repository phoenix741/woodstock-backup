<template>
  <v-btn class="ml-1" color="primary" variant="text"
    >Launch backup
    <v-dialog v-model="dialog" activator="parent" width="auto">
      <v-card v-if="dialogState == ProgressDialogState.Waiting">
        <v-card-text class="justify-center">
          <h2 class="text-h5 mb-6">Launch a manual backup</h2>

          <p class="mb-4 text-medium-emphasis text-body-2">
            To launch a manual backup, you must ensure that your device is online.
          </p>
        </v-card-text>
        <v-card-actions class="justify-end">
          <v-btn color="primary" variant="text" @click="dialog = false"> Cancel </v-btn>
          <v-spacer></v-spacer>
          <v-btn color="primary" rounded variant="flat" @click="createBackup()"> Continue </v-btn>
        </v-card-actions>
      </v-card>
      <v-card v-else-if="dialogState == ProgressDialogState.InProgress">
        <v-card-text>
          <v-progress-linear indeterminate></v-progress-linear>
        </v-card-text>
      </v-card>
      <v-card v-else-if="dialogState == ProgressDialogState.Success">
        <v-card-text class="justify-center">
          <h2 class="text-h5 mb-6">Backup is launch successfully</h2>

          <p class="mb-4 text-medium-emphasis text-body-2">
            Your backup is launch successfully. You can check the status of your tasks (<router-link
              to="/tasks/active"
              >{{ jobId }}</router-link
            >) in the task list.
          </p>
        </v-card-text>
        <v-card-actions class="justify-end">
          <v-btn color="primary" rounded variant="flat" to="/tasks/active"> Close </v-btn>
        </v-card-actions>
      </v-card>
      <v-card v-else-if="dialogState == ProgressDialogState.Error">
        <v-card-text class="justify-center">
          <h2 class="text-h5 mb-6">Backup can't be launch</h2>

          <p class="mb-4 text-medium-emphasis text-body-2">The backup can't be launch.</p>

          <p>
            {{ errorMessage }}
          </p>
        </v-card-text>
        <v-card-actions class="justify-end">
          <v-btn color="primary" rounded variant="flat" @click="dialog = false"> Close </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-btn>
</template>

<script lang="ts" setup>
import { CreateBackupDocument } from '@/generated/graphql';
import { useMutation } from '@vue/apollo-composable';
import { ref } from 'vue';

enum ProgressDialogState {
  Waiting,
  InProgress,
  Success,
  Error,
}

const props = defineProps<{
  deviceId: string;
}>();

const dialog = ref(false);
const dialogState = ref(ProgressDialogState.Waiting);

const jobId = ref('');
const errorMessage = ref('');

const { mutate } = useMutation(CreateBackupDocument);

const createBackup = async () => {
  dialogState.value = ProgressDialogState.InProgress;

  const { data, errors } =
    (await mutate({
      hostname: props.deviceId,
    })) ?? {};

  if (data?.createBackup?.id) {
    jobId.value = data.createBackup.id;
    dialogState.value = ProgressDialogState.Success;
  }

  const error = errors?.join(', ');
  if (error) {
    errorMessage.value = error;
    dialogState.value = ProgressDialogState.Error;
  }
};
</script>
