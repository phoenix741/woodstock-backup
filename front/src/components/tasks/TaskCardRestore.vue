<template>
  <AbstractTaskCard :title="title" :subtitle="executionStateMessage" icon="cloud-upload"
    :progress-percent="progress?.restoreProgression?.percent ?? 0" :progress-message="progressMessage"
    :error-message="errorMessage" :backup-error-state="!!progress?.restoreErrorState" :expanded="expanded">
    <template #tags>
      <v-chip size="small" class="ma-1" color="blue" variant="outlined">
        <v-icon start size="small">mdi-file-multiple</v-icon>
        {{ toNumber(progress?.restoreProgression?.fileCount) }} files
      </v-chip>
      <v-chip size="small" class="ma-1" color="green" variant="outlined">
        <v-icon start size="small">mdi-harddisk</v-icon>
        {{ filesize(progress?.restoreProgression?.fileSize ?? 0n) }}
      </v-chip>
      <v-chip size="small" class="ma-1" color="orange" variant="outlined">
        <v-icon start size="small">mdi-zip-box</v-icon>
        {{ filesize(progress?.restoreProgression?.compressedFileSize ?? 0n) }}
      </v-chip>
      <v-chip v-if="(progress?.restoreProgression?.errorCount ?? 0) > 0" size="small" class="ma-1" color="error"
        variant="outlined">
        <v-icon start size="small">mdi-alert</v-icon>
        {{ progress?.restoreProgression?.errorCount }} errors
      </v-chip>
      <v-chip v-if="progress?.restoreErrorState" size="small" class="ma-1" color="error" variant="flat">
        <v-icon start size="small">mdi-alert-circle</v-icon>
        FAILED
      </v-chip>
    </template>
  </AbstractTaskCard>
</template>

<script lang="ts" setup>
import { computed } from 'vue';
import type { RestoreTaskStateFragment, JobRestoreDataFragment } from '@/generated/graphql';
import { RestoreExecutionState, RestoreErrorState } from '@/generated/graphql';
import { toDateTime, toNumber } from '@/components/hosts/hosts.utils';
import filesize from '@/utils/filesize';
import AbstractTaskCard from './AbstractTaskCard.vue';

const { data, progress, expanded } = defineProps<{
  data: JobRestoreDataFragment;
  progress: RestoreTaskStateFragment | undefined | null;
  expanded?: boolean;
}>();

// Computed properties for AbstractTaskCard
const title = computed(() => `Restore ${data.host}${data.startDate ? ' - ' + toDateTime(data.startDate) : ''}`);

const progressMessage = computed(() => {
  if ((progress?.restoreProgression?.speed ?? 0) > 0) {
    return ` (${filesize(progress?.restoreProgression.speed ?? 0n)}/s)`;
  }
  return '';
});

const errorMessage = computed(() => {
  const errors = [];

  switch (progress?.restoreErrorState) {
    case RestoreErrorState.AuthenticationError:
      errors.push('Authentication failed');
      break;
    case RestoreErrorState.PreparationError:
      errors.push('Preparation failed');
      break;
    case RestoreErrorState.RestoreError:
      errors.push('Restore operation failed');
      break;
    case RestoreErrorState.Unknown:
    default:
      errors.push('Unknown error occurred');
  }

  if (progress?.restoreErrorMessage) {
    errors.push(progress.restoreErrorMessage);
  }

  return errors.join(' - ');
});

// Computed properties for UI display
const executionStateMessage = computed(() => {
  switch (progress?.restoreExecutionState) {
    case RestoreExecutionState.Waiting:
      return 'Waiting...';
    case RestoreExecutionState.Authentication:
      return 'Authenticating...';
    case RestoreExecutionState.Preparation:
      return 'Preparing restore...';
    case RestoreExecutionState.Restoring:
      return 'Restoring files...';
    case RestoreExecutionState.Completed:
      return 'Completed';
    default:
      return 'In progress...';
  }
});

</script>
