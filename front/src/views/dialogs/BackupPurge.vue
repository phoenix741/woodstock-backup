<template>
  <v-btn class="ml-1" color="warning" variant="text"
    >Purge
    <v-dialog v-model="dialog" activator="parent" width="auto">
      <v-card v-if="dialogState == ProgressDialogState.Waiting">
        <v-card-text class="justify-center">
          <h2 class="text-h5 mb-6">Purge surplus backups</h2>

          <p class="mb-4 text-medium-emphasis text-body-2">
            This will immediately delete all backups marked as <strong>Surplus</strong> for this host, according to the
            configured retention policy.
          </p>
          <p class="mb-4 text-medium-emphasis text-body-2">This action cannot be undone.</p>
        </v-card-text>
        <v-card-actions class="justify-end">
          <v-btn color="primary" variant="text" @click="dialog = false"> Cancel </v-btn>
          <v-spacer></v-spacer>
          <v-btn color="warning" rounded variant="flat" @click="purge()"> Purge </v-btn>
        </v-card-actions>
      </v-card>
      <v-card v-else-if="dialogState == ProgressDialogState.InProgress">
        <v-card-text>
          <v-progress-linear indeterminate></v-progress-linear>
        </v-card-text>
      </v-card>
      <v-card v-else-if="dialogState == ProgressDialogState.Success">
        <v-card-text class="justify-center">
          <h2 class="text-h5 mb-6">Purge enqueued successfully</h2>

          <p class="mb-4 text-medium-emphasis text-body-2">
            The surplus backups are being removed. You can follow progress in the
            <router-link to="/tasks/started">task list</router-link>.
          </p>
        </v-card-text>
        <v-card-actions class="justify-end">
          <v-btn color="primary" rounded variant="flat" @click="dialog = false"> Close </v-btn>
        </v-card-actions>
      </v-card>
      <v-card v-else-if="dialogState == ProgressDialogState.Error">
        <v-card-text class="justify-center">
          <h2 class="text-h5 mb-6">Purge failed</h2>

          <p class="mb-4 text-medium-emphasis text-body-2">The purge could not be started.</p>

          <p>{{ errorMessage }}</p>
        </v-card-text>
        <v-card-actions class="justify-end">
          <v-btn color="primary" rounded variant="flat" @click="dialog = false"> Close </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-btn>
</template>

<script lang="ts" setup>
import { PurgeRetentionDocument } from '@/generated/graphql';
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
const errorMessage = ref('');

const { mutate } = useMutation(PurgeRetentionDocument);

const purge = async () => {
  dialogState.value = ProgressDialogState.InProgress;

  const { data, errors } =
    (await mutate({
      hostname: props.deviceId,
    })) ?? {};

  if (data?.purgeRetention?.id) {
    dialogState.value = ProgressDialogState.Success;
  }

  const error = errors?.join(', ');
  if (error) {
    errorMessage.value = error;
    dialogState.value = ProgressDialogState.Error;
  }
};
</script>
