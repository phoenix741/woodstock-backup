<template>
  <AbstractTaskCard
    :title="title"
    :subtitle="globalProgressText"
    icon="delete"
    :progress-percent="globalProgress"
    :error-message="errorMessage"
    :backup-error-state="hasError"
    :expanded="expanded"
  >
  </AbstractTaskCard>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import {
  RemoveErrorState,
  RemoveExecutionState,
  type JobRemoveDataFragment,
  type RemoveTaskStateFragment,
} from '@/generated/graphql';
import { toDateTime, toNumber } from '@/components/hosts/hosts.utils';
import AbstractTaskCard from './AbstractTaskCard.vue';

const { data, progress, expanded } = defineProps<{
  data: JobRemoveDataFragment;
  progress: RemoveTaskStateFragment | undefined | null;
  expanded?: boolean;
}>();

const title = computed(
  () =>
    `Remove Backup ${data.host} #${toNumber(data.number)}${data.startDate ? ' - ' + toDateTime(data.startDate) : ''}`,
);

const hasError = computed(() => progress?.removeErrorState !== null && progress?.removeErrorState !== undefined);

const globalProgress = computed(() => {
  switch (progress?.removeExecutionState) {
    case RemoveExecutionState.Waiting:
      return 5;
    case RemoveExecutionState.AddReferencesToPool:
      return 25;
    case RemoveExecutionState.RemovingRefcnt:
      return 50;
    case RemoveExecutionState.RemovingBackup:
      return 75;
    case RemoveExecutionState.Completed:
      return 100;
    default:
      return 0;
  }
});

const globalProgressText = computed((): string => {
  if (!progress) return 'Loading...';

  switch (progress?.removeExecutionState) {
    case RemoveExecutionState.Waiting:
      return 'Waiting to start removal...';
    case RemoveExecutionState.AddReferencesToPool:
      return 'Adding references to pool...';
    case RemoveExecutionState.RemovingRefcnt:
      return 'Removing reference counts...';
    case RemoveExecutionState.RemovingBackup:
      return 'Removing backup data...';
    case RemoveExecutionState.Completed:
      return 'Backup removed successfully';
    default:
      return 'Unknown state';
  }
});

const errorMessage = computed(() => {
  const errors = [];

  // The error state uses the same RemoveExecutionState enum to indicate which phase failed
  switch (progress?.removeErrorState) {
    case RemoveErrorState.AddReferencesToPoolError:
      errors.push('Failed to add references to pool');
      break;
    case RemoveErrorState.RefcntRemovalError:
      errors.push('Failed to remove reference counts');
      break;
    case RemoveErrorState.BackupRemovalError:
      errors.push('Failed to remove backup data');
      break;
    default:
      errors.push('Unknown error occurred during removal');
  }
  if (progress?.removeErrorMessage) {
    errors.push(progress.removeErrorMessage);
  }

  return errors.join(', ');
});
</script>
