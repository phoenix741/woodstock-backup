<template>
  <v-btn block
    >{{ buttonAction }}
    <v-dialog v-model="dialog" activator="parent" width="700px" height="500px">
      <v-card v-if="dialogState == ProgressDialogState.Waiting">
        <v-card-text class="justify-center">
          <h2 class="text-h5 mb-6">{{ action }}</h2>

          <p class="mb-4 text-medium-emphasis text-body-2">
            {{ description }}

            <template v-if="withFix">
              <br />
              <br />
              <v-checkbox v-model="fixErrors" label="Fix errors"></v-checkbox>
            </template>
          </p>
        </v-card-text>
        <v-card-actions class="justify-end">
          <v-btn color="primary" variant="text" @click="dialog = false"> Cancel </v-btn>
          <v-spacer></v-spacer>
          <v-btn color="primary" rounded variant="flat" @click="launchJob()"> Continue </v-btn>
        </v-card-actions>
      </v-card>
      <v-card v-else-if="dialogState == ProgressDialogState.InProgress">
        <v-card-text>
          <h2 class="text-h5 mb-6">{{ action }}</h2>

          <v-progress-linear indeterminate></v-progress-linear>
        </v-card-text>
      </v-card>
      <v-card v-else-if="dialogState == ProgressDialogState.Success">
        <v-card-text class="justify-center">
          <h2 class="text-h5 mb-6">{{ action }}</h2>

          <p class="mb-4 text-medium-emphasis text-body-2">The job have been launch successfully.</p>
        </v-card-text>
        <v-card-actions class="justify-end">
          <v-btn color="primary" rounded variant="flat" @click="dialog = false"> Close </v-btn>
        </v-card-actions>
      </v-card>
      <v-card v-else-if="dialogState == ProgressDialogState.Error">
        <v-card-text class="justify-center">
          <h2 class="text-h5 mb-6">{{ action }}</h2>

          <p class="mb-4 text-medium-emphasis text-body-2">The job can't be launch.</p>

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
import { JobResponse } from '@/generated/graphql';
import { MutateFunction } from '@vue/apollo-composable';
import { ref } from 'vue';

enum ProgressDialogState {
  Waiting,
  InProgress,
  Success,
  Error,
}

const props = defineProps<{
  buttonAction: string;

  action: string;
  description: string;
  withFix: boolean;

  mutate: MutateFunction<JobResponse, any>;
}>();

const dialog = ref(false);
const dialogState = ref(ProgressDialogState.Waiting);

const jobId = ref('');
const errorMessage = ref('');

const fixErrors = ref(false);

const launchJob = async () => {
  dialogState.value = ProgressDialogState.InProgress;
  const variables = props.withFix ? fixErrors.value : {};
  const { data, errors } = (await props.mutate(variables)) ?? {};

  if (data?.id) {
    jobId.value = data.id;
    dialogState.value = ProgressDialogState.Success;
  }

  const error = errors?.join(', ');
  if (error) {
    errorMessage.value = error;
    dialogState.value = ProgressDialogState.Error;
  }
};
</script>
