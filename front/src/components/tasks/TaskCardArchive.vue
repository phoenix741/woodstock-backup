<template>
  <AbstractTaskCard
    :title="title"
    :subtitle="subtitle"
    icon="archive"
    :progress-percent="progressPercent"
    :progress-message="progressMessage"
    :error-message="errorMessage"
    :backup-error-state="hasError"
    :expanded="expanded"
  >
    <template #tags>
      <v-chip size="small" class="ma-1" color="blue" variant="outlined">
        <v-icon start size="small">mdi-cog</v-icon>
        {{ data.profileName }}
      </v-chip>
      <v-chip size="small" class="ma-1" color="default" variant="outlined">
        <v-icon start size="small">mdi-server</v-icon>
        {{ hostsSummary }}
      </v-chip>
      <v-chip size="small" class="ma-1" color="blue" variant="outlined">
        <v-icon start size="small">mdi-file-multiple</v-icon>
        {{ toNumber(progress?.fileCount ?? 0) }} files
      </v-chip>
      <v-chip size="small" class="ma-1" color="green" variant="outlined">
        <v-icon start size="small">mdi-harddisk</v-icon>
        {{ filesize(progress?.progressCurrent ?? 0n) }}
      </v-chip>
      <v-chip size="small" class="ma-1" color="orange" variant="outlined">
        <v-icon start size="small">mdi-zip-box</v-icon>
        {{ progress?.archiveSize ? filesize(progress.archiveSize) : '—' }}
      </v-chip>
      <v-chip v-if="failedHosts.length > 0" size="small" class="ma-1" color="warning" variant="outlined">
        <v-icon start size="small">mdi-alert</v-icon>
        {{ failedHosts.length }} host(s) failed: {{ failedHosts.join(', ') }}
      </v-chip>
    </template>

    <template v-if="hostStates.length > 0" #details>
      <v-container>
        <v-row>
          <v-col cols="12">
            <h4 class="text-subtitle-2 mb-3">
              <v-icon size="small" class="mr-2">mdi-server</v-icon>
              Hosts Progress
            </h4>
            <div v-for="host in hostStates" :key="host.hostname" class="mb-4">
              <div class="d-flex align-center mb-2">
                <v-chip :color="hostStateColor(host.executionState)" variant="outlined" class="mr-2">
                  <v-icon start size="small">{{ hostStateIcon(host.executionState) }}</v-icon>
                  {{ host.hostname }}
                </v-chip>
                <span class="text-caption text-secondary ml-2">
                  {{ hostStateText(host.executionState) }}
                </span>
              </div>

              <v-progress-linear
                v-if="host.executionState === ArchiveHostExecutionState.InProgress"
                :color="hostStateColor(host.executionState)"
                :model-value="host.percent"
                height="20"
                class="mb-2"
              >
                <small>
                  {{ toPercent(host.percent) }} — {{ filesize(host.progressCurrent) }} / {{ filesize(host.progressMax) }}
                  — {{ toNumber(host.fileCount) }} file(s)
                </small>
              </v-progress-linear>

              <div v-else-if="host.executionState === ArchiveHostExecutionState.Success" class="text-caption text-secondary">
                <v-icon size="small">mdi-harddisk</v-icon>
                {{ filesize(host.progressCurrent) }}, {{ toNumber(host.fileCount) }} file(s) archived
                <template v-if="host.archiveSize != null && host.archiveSize !== host.progressCurrent">
                  ({{ filesize(host.archiveSize) }} archive)
                </template>
              </div>

              <div v-else-if="host.executionState === ArchiveHostExecutionState.Waiting" class="text-caption text-secondary">
                Waiting to start...
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
import {
  ArchiveHostExecutionState,
  JobStatus,
  type ArchiveTaskStateFragment,
  type JobArchiveDataFragment,
} from '@/generated/graphql';
import { toPercent, toNumber } from '@/components/hosts/hosts.utils';
import filesize from '@/utils/filesize';
import AbstractTaskCard from './AbstractTaskCard.vue';

const { data, progress, status, failedReason, expanded } = defineProps<{
  data: JobArchiveDataFragment;
  progress?: ArchiveTaskStateFragment | null;
  status: JobStatus;
  failedReason?: string | null;
  expanded?: boolean;
}>();

const title = computed(() => `Archive (${data.profileName})`);

const hostsSummary = computed(() => {
  const total = progress?.hostsTotal ?? data.hostnames.length;
  const done = progress?.hostsDone ?? 0;
  return `${done}/${total} host(s)`;
});

const failedHosts = computed(() => progress?.failedHosts ?? []);
const hostStates = computed(() => progress?.hostStates ?? []);

// The job itself is only marked Failed for a structural problem (e.g. it
// couldn't even start) — a per-host failure inside an otherwise-successful
// run shows up as the warning chip above instead, so it doesn't hijack the
// whole progress display.
const hasError = computed(() => status === JobStatus.Failed);

// Real, byte-based progress once the job has started publishing it; falls
// back to a coarse guess before the first progress update arrives.
const progressPercent = computed(() => {
  if (progress) return progress.percent;
  switch (status) {
    case JobStatus.Created:
      return 0;
    case JobStatus.Completed:
    case JobStatus.Failed:
      return 100;
    default:
      return 0;
  }
});

// Mirrors TaskCardBackup.vue's progressMessage (which host is running, and
// at what throughput) rather than just the speed alone, since the archive
// run is sequential across hosts and "which host" is the more useful detail
// here.
const progressMessage = computed(() => {
  if (!progress?.currentHost) return undefined;
  const speed = (progress.speed ?? 0) > 0 ? ` · ${filesize(progress.speed)}/s` : '';
  return `archiving ${progress.currentHost}...${speed}`;
});

const subtitle = computed(() => {
  switch (status) {
    case JobStatus.Created:
      return 'Waiting to start archiving...';
    case JobStatus.Started:
      return 'Archiving in progress...';
    case JobStatus.Completed:
      return failedHosts.value.length > 0 ? 'Archive completed with failures' : 'Archive completed';
    case JobStatus.Failed:
      return 'Archive failed';
    default:
      return 'Unknown state';
  }
});

const errorMessage = computed(() => failedReason ?? 'Unknown error occurred during archiving');

function hostStateColor(state: ArchiveHostExecutionState): string {
  switch (state) {
    case ArchiveHostExecutionState.InProgress:
      return 'primary';
    case ArchiveHostExecutionState.Waiting:
      return 'grey';
    case ArchiveHostExecutionState.Success:
      return 'success';
    case ArchiveHostExecutionState.Failed:
      return 'error';
    default:
      return 'grey';
  }
}

function hostStateIcon(state: ArchiveHostExecutionState): string {
  switch (state) {
    case ArchiveHostExecutionState.InProgress:
      return 'mdi-progress-upload';
    case ArchiveHostExecutionState.Waiting:
      return 'mdi-clock-outline';
    case ArchiveHostExecutionState.Success:
      return 'mdi-check-circle';
    case ArchiveHostExecutionState.Failed:
      return 'mdi-alert-circle';
    default:
      return 'mdi-server';
  }
}

function hostStateText(state: ArchiveHostExecutionState): string {
  switch (state) {
    case ArchiveHostExecutionState.InProgress:
      return 'Archiving in progress';
    case ArchiveHostExecutionState.Waiting:
      return 'Waiting to start';
    case ArchiveHostExecutionState.Success:
      return 'Completed successfully';
    case ArchiveHostExecutionState.Failed:
      return 'Failed';
    default:
      return 'Unknown state';
  }
}
</script>
