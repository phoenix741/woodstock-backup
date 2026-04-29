<template>
  <AbstractTaskCard
    :title="title"
    :subtitle="subtitle"
    icon="broom"
    :progress-percent="progressPercent"
    :progress-message="progressMessage"
    :error-message="errorMessage"
    :backup-error-state="hasError"
    :expanded="expanded"
  >
    <template #tags>
      <v-chip v-if="progress?.cleanerProgress" size="small" class="ma-1" color="blue" variant="outlined">
        <v-icon start size="small">mdi-progress-check</v-icon>
        {{ toNumber(progress.cleanerProgress.progressCurrent) }} / {{ toNumber(progress.cleanerProgress.progressMax) }}
      </v-chip>
      <v-chip v-if="progress?.cleanerProgress" size="small" class="ma-1" color="green" variant="outlined">
        <v-icon start size="small">mdi-harddisk</v-icon>
        {{ filesize(progress.cleanerProgress.fileSize) }}
      </v-chip>
      <v-chip v-if="progress?.cleanerProgress" size="small" class="ma-1" color="orange" variant="outlined">
        <v-icon start size="small">mdi-zip-box</v-icon>
        {{ filesize(progress.cleanerProgress.compressedFileSize) }}
      </v-chip>
      <v-chip v-if="hasError" size="small" class="ma-1" color="error" variant="flat">
        <v-icon start size="small">mdi-alert-circle</v-icon>
        FAILED
      </v-chip>
    </template>
  </AbstractTaskCard>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import {
  CleanerExecutionState,
  CleanerErrorState,
  type JobCleanupDataFragment,
  type CleanerTaskStateFragment,
} from '@/generated/graphql';
import AbstractTaskCard from './AbstractTaskCard.vue';
import { toNumber } from '../hosts/hosts.utils';
import filesize from '@/utils/filesize';

const { data, progress, expanded } = defineProps<{
  data: JobCleanupDataFragment;
  progress: CleanerTaskStateFragment | undefined | null;
  expanded?: boolean;
}>();

const title = computed(() => `Pool Cleanup ${data.target ? `- ${data.target}` : ''}`);

const subtitle = computed(() => {
  if (hasError.value) {
    return 'Cleanup failed with error';
  }

  switch (progress?.cleanerExecutionState) {
    case CleanerExecutionState.Waiting:
      return 'Waiting in queue';
    case CleanerExecutionState.Initialization:
      return 'Initializing cleanup process';
    case CleanerExecutionState.ApplyingRefcnt:
      return 'Applying reference counts';
    case CleanerExecutionState.Cleaning:
      return 'Cleaning unused data';
    case CleanerExecutionState.Completed:
      return 'Cleanup completed successfully';
    default:
      return 'Unknown state';
  }
});

const progressPercent = computed(() => {
  if (progress?.cleanerExecutionState === CleanerExecutionState.Completed) {
    return 100;
  }

  if ((progress?.cleanerProgress?.progressMax ?? 0) > 0) {
    return ((progress?.cleanerProgress?.progressCurrent ?? 0) / (progress?.cleanerProgress?.progressMax ?? 1)) * 100;
  }
  return 0;
});

const progressMessage = computed(() => {
  if (!progress) return 'Loading...';

  switch (progress.cleanerExecutionState) {
    case CleanerExecutionState.Waiting:
      return 'Waiting to start cleanup...';
    case CleanerExecutionState.Initialization:
      return 'Initializing cleanup process...';
    case CleanerExecutionState.ApplyingRefcnt:
      return 'Applying reference counts...';
    case CleanerExecutionState.Cleaning:
      return 'Cleaning unused data...';
    case CleanerExecutionState.Completed:
      return 'Pool cleanup completed successfully';
    default:
      return 'Unknown state';
  }
});

const errorMessage = computed(() => {
  const message = [];

  switch (progress?.cleanerErrorState) {
    case CleanerErrorState.InitializationError:
      message.push('Failed to initialize cleanup process');
      break;
    case CleanerErrorState.ApplyingRefcntError:
      message.push('Failed to apply reference counts');
      break;
    case CleanerErrorState.CleaningError:
      message.push('Failed to clean unused data');
      break;
    case CleanerErrorState.Unknown:
    default:
      message.push('Unknown cleanup error occurred');
      break;
  }
  if (progress?.cleanerErrorMessage) {
    message.push(progress.cleanerErrorMessage);
  }

  return message.join(' - ');
});

const hasError = computed(() => !!progress?.cleanerErrorState || !!progress?.cleanerErrorMessage);
</script>
