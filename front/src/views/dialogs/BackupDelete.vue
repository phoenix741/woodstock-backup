<template>
  <v-btn class="ml-12" color="secondary" variant="text"
    >Delete
    <v-dialog v-model="dialog" activator="parent" width="auto">
      <v-card>
        <v-card-text class="justify-center">
          <h2 class="text-h5 mb-6">Removing backup</h2>

          <p class="mb-4 text-medium-emphasis text-body-2" v-if="dialogState == ProgressDialogState.Waiting">
            Are you sure to remove this backups:
          </p>
          <p class="mb-4 text-medium-emphasis text-body-2">
            <v-table>
              <thead>
                <tr>
                  <th class="text-left">Backup number</th>
                  <th class="text-left">Result</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="n in backupNumbers" :key="n">
                  <td>{{ n }}</td>
                  <td v-if="results.get(n)?.jobId">Job n°{{ results.get(n)?.jobId }}</td>
                  <td v-else-if="results.get(n)?.message">Error: {{ results.get(n)?.message }}</td>
                  <td v-else-if="dialogState == ProgressDialogState.InProgress">
                    <v-progress-circular indeterminate></v-progress-circular>
                  </td>
                  <td v-else></td>
                </tr>
              </tbody>
            </v-table>
          </p>
        </v-card-text>
        <v-card-actions class="justify-end" v-if="dialogState == ProgressDialogState.Waiting">
          <v-btn color="primary" variant="text" @click="dialog = false"> Cancel </v-btn>
          <v-spacer></v-spacer>
          <v-btn color="primary" rounded variant="flat" @click="removeBackup()"> Continue </v-btn>
        </v-card-actions>
        <v-card-actions class="justify-end" v-if="dialogState == ProgressDialogState.Result">
          <v-btn color="primary" variant="text" @click="dialog = false"> Close </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-btn>
</template>

<script lang="ts" setup>
import { RemoveBackupDocument } from '@/generated/graphql';
import { useMutation } from '@vue/apollo-composable';
import { ref, reactive } from 'vue';

enum ProgressDialogState {
  Waiting,
  InProgress,
  Result,
}

const props = defineProps<{
  deviceId: string;
  backupNumbers: number[];
}>();

const dialog = ref(false);
const dialogState = ref(ProgressDialogState.Waiting);

const results = reactive(new Map<number, { message?: string; jobId?: string }>());

const { mutate } = useMutation(RemoveBackupDocument);

const removeBackup = async () => {
  dialogState.value = ProgressDialogState.InProgress;
  for (const backup of props.backupNumbers) {
    const { data, errors } =
      (await mutate({
        hostname: props.deviceId,
        number: backup,
      })) ?? {};

    if (data?.removeBackup?.id) {
      results.set(backup, { jobId: data.removeBackup.id });
    }
    const error = errors?.join(', ');
    if (error) {
      results.set(backup, { message: error });
    }
  }
  dialogState.value = ProgressDialogState.Result;
};
</script>
