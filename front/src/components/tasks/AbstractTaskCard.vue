<template>
  <v-sheet rounded="lg">
    <v-expansion-panels v-model="localExpanded">
      <v-expansion-panel
        :value="0"
        :hide-actions="!$slots.details && !canCancel"
        :readonly="!$slots.details && !canCancel"
      >
        <v-expansion-panel-title>
          <v-container>
            <v-row no-gutters>
              <v-col cols="8">
                <span class="font-weight-light">
                  <v-icon start size="small">mdi-{{ icon }}</v-icon>
                  {{ title }}
                </span>
              </v-col>
              <v-col cols="4">
                <div class="text-right font-weight-light">{{ subtitle }}</div>
              </v-col>
            </v-row>
            <!-- Global Progress -->
            <v-row class="pt-5" no-gutters>
              <v-col cols="12">
                <v-progress-linear
                  :color="progressColor"
                  striped
                  :model-value="progressPercent"
                  :indeterminate="false"
                  height="25"
                >
                  <strong v-if="backupErrorState"> ERROR: {{ errorMessage }} </strong>
                  <strong v-else>
                    {{ toPercent(progressPercent) }}
                    <template v-if="progressMessage">- {{ progressMessage }}</template>
                  </strong>
                </v-progress-linear>
              </v-col>
            </v-row>
            <!-- Backup Details -->
            <v-row class="pt-3" no-gutters>
              <v-col cols="12">
                <div class="text-caption text-grey">
                  <slot name="tags"></slot>
                </div>
              </v-col>
            </v-row>
          </v-container>
        </v-expansion-panel-title>

        <v-expansion-panel-text>
          <slot name="details"></slot>

          <div v-if="canCancel" class="d-flex align-center justify-end mt-2">
            <span v-if="cancelFeedback" class="text-caption text-grey mr-3">{{ cancelFeedback }}</span>
            <v-btn
              variant="tonal"
              color="error"
              prepend-icon="mdi-stop-circle-outline"
              :loading="cancelling"
              @click="confirmDialog = true"
            >
              Cancel task
            </v-btn>
          </div>
        </v-expansion-panel-text>
      </v-expansion-panel>
    </v-expansion-panels>

    <v-dialog v-model="confirmDialog" width="480">
      <v-card>
        <v-card-title class="text-h6">Cancel this task?</v-card-title>
        <v-card-text>
          {{ cancelWarning ?? 'This will stop the task in progress.' }}
        </v-card-text>
        <v-card-actions class="justify-end">
          <v-btn variant="text" @click="confirmDialog = false">Keep running</v-btn>
          <v-btn color="error" variant="flat" :loading="cancelling" @click="onCancel">Cancel task</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-sheet>
</template>

<script lang="ts" setup>
import { computed, ref } from 'vue';
import { toPercent } from '@/components/hosts/hosts.utils';
import { CancelJobDocument, JobStatus } from '@/generated/graphql';
import { useMutation } from '@vue/apollo-composable';

const props = defineProps<{
  title: string;
  subtitle?: string;
  icon: string;
  progressPercent: number;
  progressMessage?: string;
  errorMessage?: string;
  backupErrorState?: boolean;
  /** Set once the task was stopped by a user cancel — renders the progress
   * bar in a neutral color instead of the "in progress" primary one, without
   * treating it as an error. */
  cancelled?: boolean;
  /** If true, the panel is expanded on mount (e.g. running task). */
  expanded?: boolean;
  /** Job id to target with the cancel mutation. Only Backup/Restore/Archive/
   * Fsck cards pass this — omitting it (the default for other job kinds)
   * hides the Cancel button entirely. */
  jobId?: string;
  /** Only Created/Started jobs can still be cancelled. */
  jobStatus?: JobStatus;
  /** Job-kind-specific explanation of what cancelling actually does, shown
   * in the confirmation dialog (e.g. "files already restored stay in
   * place"). Falls back to a generic message if omitted. */
  cancelWarning?: string;
}>();

// Initialized from prop but allows user interaction afterwards
const localExpanded = ref<number | undefined>(props.expanded ? 0 : undefined);

const progressColor = computed(() => {
  if (props.backupErrorState) {
    return 'error';
  }
  if (props.cancelled) {
    return 'grey';
  }
  return 'primary';
});

const canCancel = computed(
  () => !!props.jobId && (props.jobStatus === JobStatus.Created || props.jobStatus === JobStatus.Started),
);

const { mutate, loading: cancelling } = useMutation(CancelJobDocument);

const confirmDialog = ref(false);

// `cancelJob` resolves to `false` (not an Apollo error) when the job already
// reached a terminal status between render and click — a real, if narrow,
// race the button must surface instead of silently doing nothing.
const cancelFeedback = ref<string | undefined>(undefined);

async function onCancel() {
  if (!props.jobId) return;
  cancelFeedback.value = undefined;
  try {
    const result = await mutate({ taskId: props.jobId });
    if (result?.data?.cancelJob !== true) {
      cancelFeedback.value = 'Already finished';
      setTimeout(() => (cancelFeedback.value = undefined), 3000);
    }
  } catch {
    cancelFeedback.value = 'Cancel failed';
    setTimeout(() => (cancelFeedback.value = undefined), 3000);
  } finally {
    confirmDialog.value = false;
  }
}
</script>
