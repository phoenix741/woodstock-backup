<template>
  <v-btn color="primary" variant="flat"
    >Run now
    <v-dialog v-model="dialog" activator="parent" width="500">
      <v-card v-if="dialogState === RunDialogState.Waiting">
        <v-card-text>
          <h2 class="text-h5 mb-6">Run "{{ profileName }}" now</h2>

          <p class="mb-4 text-medium-emphasis text-body-2">
            Runs immediately, whether or not the profile is enabled or currently due.
          </p>

          <v-text-field
            v-model="hostOverride"
            label="Host (optional — leave empty to use the profile's selection)"
            clearable
            autofocus
          />
        </v-card-text>
        <v-card-actions class="justify-end">
          <v-btn color="primary" variant="text" @click="dialog = false"> Cancel </v-btn>
          <v-spacer></v-spacer>
          <v-btn color="primary" rounded variant="flat" @click="runArchive()"> Run </v-btn>
        </v-card-actions>
      </v-card>
      <v-card v-else-if="dialogState === RunDialogState.InProgress">
        <v-card-text>
          <v-progress-linear indeterminate></v-progress-linear>
        </v-card-text>
      </v-card>
      <v-card v-else-if="dialogState === RunDialogState.Success">
        <v-card-text class="justify-center">
          <h2 class="text-h5 mb-6">Archive launched</h2>

          <p class="mb-4 text-medium-emphasis text-body-2">
            You can check its status in the <router-link to="/tasks">task list</router-link>.
          </p>
        </v-card-text>
        <v-card-actions class="justify-end">
          <v-btn color="primary" rounded variant="flat" @click="dialog = false"> Close </v-btn>
        </v-card-actions>
      </v-card>
      <v-card v-else-if="dialogState === RunDialogState.Error">
        <v-card-text class="justify-center">
          <h2 class="text-h5 mb-6">Archive can't be launched</h2>

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
import { RunArchiveDocument } from '@/generated/graphql';
import { useMutation } from '@vue/apollo-composable';
import { ref } from 'vue';

const props = defineProps<{
  profileName: string;
}>();

enum RunDialogState {
  Waiting,
  InProgress,
  Success,
  Error,
}

const dialog = ref(false);
const dialogState = ref(RunDialogState.Waiting);

const hostOverride = ref('');
const errorMessage = ref('');

const { mutate } = useMutation(RunArchiveDocument);

const runArchive = async () => {
  dialogState.value = RunDialogState.InProgress;

  const { data, errors } =
    (await mutate({
      profile: props.profileName,
      host: hostOverride.value || null,
    })) ?? {};

  if (data?.runArchive?.jobIds && data.runArchive.jobIds.length > 0) {
    dialogState.value = RunDialogState.Success;
    return;
  }

  errorMessage.value = errors?.join(', ') || 'No host matched the profile’s selection — nothing was launched.';
  dialogState.value = RunDialogState.Error;
};
</script>
