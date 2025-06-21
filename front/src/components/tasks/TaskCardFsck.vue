<template>
  <AbstractTaskCard :title="title" :subtitle="subtitle" icon="shield-search" :progress-percent="progressPercent"
    :progress-message="progressMessage" :error-message="errorMessage" :backup-error-state="hasError">
    <template #tags>
      <v-chip v-if="progress?.refcntProgression" size="small" class="ma-1" color="blue" variant="outlined">
        <v-icon start size="small">mdi-counter</v-icon>
        {{ toNumber(progress?.unusedProgression?.inRefcnt) }} refcnt
      </v-chip>
      <v-chip v-if="progress?.unusedProgression" size="small" class="ma-1" color="green" variant="outlined">
        <v-icon start size="small">mdi-delete</v-icon>
        {{ toNumber(progress?.unusedProgression?.inUnused) }} unused
      </v-chip>
      <v-chip v-if="totalErrorCount > 0" size="small" class="ma-1" color="error" variant="outlined">
        <v-icon start size="small">mdi-alert</v-icon>
        {{ totalErrorCount }} errors
      </v-chip>
      <v-chip v-if="hasError" size="small" class="ma-1" color="error" variant="flat">
        <v-icon start size="small">mdi-alert-circle</v-icon>
        FAILED
      </v-chip>
    </template>

    <template #details>
      <v-container>
        <v-row v-if="progress" class="mb-4">
          <v-col cols="12">
            <h4 class="text-subtitle-2 mb-3">
              <v-icon size="small" class="mr-2">mdi-chart-line</v-icon>
              Progress Details
            </h4>

            <!-- RefCnt Progress -->
            <div class="mb-3">
              <div class="text-subtitle-2 mb-2">Reference Count Verification</div>
              <v-progress-linear color="primary" :model-value="refcntProgressionPercent" height="20" class="mb-2">
                <small>{{ toPercent(refcntProgressionPercent) }}</small>
              </v-progress-linear>
              <div class="text-caption text-secondary">
                <span class="mr-4">
                  <v-icon size="small">mdi-counter</v-icon>
                  {{ toNumber(progress?.refcntProgression?.progressCurrent) }} / {{
                    toNumber(progress?.refcntProgression?.progressMax) }} items
                </span>
                <span class="mr-4">
                  <v-icon size="small">mdi-database</v-icon>
                  {{ toNumber(progress?.refcntProgression?.totalCount) }} total
                </span>
                <span v-if="progress?.refcntProgression?.errorCount > 0" class="mr-4 text-error">
                  <v-icon size="small">mdi-alert</v-icon>
                  {{ toNumber(progress?.refcntProgression?.errorCount) }} errors
                </span>
              </div>
            </div>

            <!-- Unused Progress -->
            <div class="mb-3">
              <div class="text-subtitle-2 mb-2">Unused Chunk Verification</div>
              <v-progress-linear color="primary" :model-value="unusedProgressionPercent" height="20" class="mb-2">
                <small>{{ toPercent(unusedProgressionPercent) }}</small>
              </v-progress-linear>
              <div class="text-caption text-secondary">
                <span class="mr-4">
                  <v-icon size="small">mdi-delete</v-icon>
                  {{ toNumber(progress?.unusedProgression?.progressCurrent) }} / {{
                    toNumber(progress?.unusedProgression?.progressMax) }} items
                </span>
                <span v-if="progress?.unusedProgression?.missing > 0" class="mr-4 text-error">
                  <v-icon size="small">mdi-help-circle</v-icon>
                  {{ toNumber(progress?.unusedProgression?.missing) }} missing
                </span>
                <span v-if="progress?.unusedProgression?.inNothing > 0" class="mr-4 text-warning">
                  <v-icon size="small">mdi-circle-outline</v-icon>
                  {{ toNumber(progress?.unusedProgression?.inNothing) }} neither in refcnt or unused
                </span>
                <span class="mr-4">
                  <v-icon size="small">mdi-counter</v-icon>
                  {{ toNumber(progress?.unusedProgression?.inRefcnt) }} in refcnt
                </span>
                <span class="mr-4">
                  <v-icon size="small">mdi-delete</v-icon>
                  {{ toNumber(progress?.unusedProgression?.inUnused) }} in unused
                </span>
              </div>
            </div>

            <!-- Chunk Progress (only if chunk verification is enabled) -->
            <div v-if="data.verifyChunks" class="mb-3">
              <div class="text-subtitle-2 mb-2">Chunk Integrity Verification</div>
              <v-progress-linear color="primary" :model-value="chunkProgressionPercent" height="20" class="mb-2">
                <small>{{ toPercent(chunkProgressionPercent) }}</small>
              </v-progress-linear>
              <div class="text-caption text-secondary">
                <span class="mr-4">
                  <v-icon size="small">mdi-cube</v-icon>
                  {{ toNumber(progress?.chunkProgression?.progressCurrent) }} / {{
                    toNumber(progress?.chunkProgression?.progressMax) }} chunks
                </span>
                <span class="mr-4">
                  <v-icon size="small">mdi-database</v-icon>
                  {{ toNumber(progress?.chunkProgression?.totalCount) }} total
                </span>
                <span v-if="progress?.chunkProgression?.errorCount > 0" class="mr-4 text-error">
                  <v-icon size="small">mdi-alert</v-icon>
                  {{ toNumber(progress?.chunkProgression?.errorCount) }} corrupted
                </span>
              </div>
            </div>
          </v-col>
        </v-row>
      </v-container>
    </template>
  </AbstractTaskCard>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { FsckExecutionState, FsckErrorState, type JobFsckDataFragment, type FsckTaskStateFragment } from '@/generated/graphql';
import AbstractTaskCard from './AbstractTaskCard.vue';
import { toNumber, toPercent } from '../hosts/hosts.utils';

const { data, progress } = defineProps<{
  data: JobFsckDataFragment;
  progress: FsckTaskStateFragment | undefined | null;
}>();

const title = computed(() => {
  let title = 'Pool Fsck';
  if (data.dryRun) title += ' (Dry Run)';
  if (data.verifyChunks) title += ' + Chunk Verification';
  return title;
});

const subtitle = computed(() => {
  if (hasError.value) {
    return 'Filesystem check failed with error';
  }

  switch (progress?.fsckExecutionState) {
    case FsckExecutionState.Waiting:
      return 'Waiting to start filesystem check';
    case FsckExecutionState.Initialization:
      return 'Initializing filesystem check';
    case FsckExecutionState.ApplyingRefcnt:
      return 'Applying reference counts';
    case FsckExecutionState.VerifyRefcnt:
      return 'Verifying reference counts';
    case FsckExecutionState.VerifyUnused:
      return 'Verifying unused chunks';
    case FsckExecutionState.VerifyChunk:
      return 'Verifying chunk integrity';
    case FsckExecutionState.Completed:
      return 'Filesystem check completed successfully';
    default:
      return 'Unknown state';
  }
});

const progressPercent = computed(() => {
  const progressMax = (progress?.refcntProgression?.progressMax ?? 0) +
    (progress?.unusedProgression?.progressMax ?? 0) +
    (progress?.chunkProgression?.progressMax ?? 0);
  const progressCurrent = (progress?.refcntProgression?.progressCurrent ?? 0) +
    (progress?.unusedProgression?.progressCurrent ?? 0) +
    (progress?.chunkProgression?.progressCurrent ?? 0);

  return progressMax > 0
    ? (progressCurrent / progressMax) * 100
    : 0;
});

const refcntProgressionPercent = computed(() => {
  if (!progress?.refcntProgression) return 0;
  return (progress.refcntProgression.progressCurrent / progress.refcntProgression.progressMax) * 100;
});

const unusedProgressionPercent = computed(() => {
  if (!progress?.unusedProgression) return 0;
  return (progress.unusedProgression.progressCurrent / progress.unusedProgression.progressMax) * 100;
});

const chunkProgressionPercent = computed(() => {
  if (!progress?.chunkProgression) return 0;
  return (progress.chunkProgression.progressCurrent / progress.chunkProgression.progressMax) * 100;
});

const progressMessage = computed(() => {
  if (!progress) return '';

  switch (progress?.fsckExecutionState) {
    case FsckExecutionState.Initialization:
      return ' - Scanning pool structure';
    case FsckExecutionState.ApplyingRefcnt:
      return ' - Updating reference counts';
    case FsckExecutionState.VerifyRefcnt:
      return ' - Checking reference consistency';
    case FsckExecutionState.VerifyUnused:
      return ' - Identifying unused chunks';
    case FsckExecutionState.VerifyChunk:
      return ' - Verifying chunk data integrity';
    default:
      return '';
  }
});

const errorMessage = computed(() => {
  const message = [];

  switch (progress?.fsckErrorState) {
    case FsckErrorState.InitializationError:
      message.push('Failed to initialize filesystem check');
      break;
    case FsckErrorState.ApplyingRefcntError:
      message.push('Failed to apply reference counts');
      break;
    case FsckErrorState.VerifyRefcntError:
      message.push('Failed to verify reference counts');
      break;
    case FsckErrorState.VerifyUnusedError:
      message.push('Failed to verify unused chunks');
      break;
    case FsckErrorState.VerifyChunkError:
      message.push('Failed to verify chunk integrity');
      break;
    case FsckErrorState.Unknown:
    default:
      message.push('Unknown filesystem check error occurred');
  }
  if (progress?.fsckErrorMessage) {
    message.push(progress.fsckErrorMessage);
  }

  return message.join(' - ');
});

const hasError = computed(() => progress?.fsckErrorState !== null && progress?.fsckErrorState !== undefined);

const totalErrorCount = computed((): number => {
  if (!progress) return 0;

  return (progress?.refcntProgression?.errorCount ?? 0) +
    (progress?.unusedProgression?.missing ?? 0) +
    (progress?.unusedProgression?.inNothing ?? 0) +
    (progress?.chunkProgression?.errorCount ?? 0);
});

</script>
