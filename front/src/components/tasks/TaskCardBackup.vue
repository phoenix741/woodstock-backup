<template>
  <AbstractTaskCard :title="title" :subtitle="executionStateMessage" icon="cloud-download"
    :progress-percent="progress?.backupProgress?.percent ?? 0" :progress-message="progressMessage"
    :error-message="errorMessage" :backup-error-state="!!progress?.backupErrorState">
    <template #tags>
      <v-chip size="small" class="ma-1" color="blue" variant="outlined">
        <v-icon start size="small">mdi-file-multiple</v-icon>
        {{ toNumber((progress?.backupProgress?.newFileCount ?? 0) + (progress?.backupProgress?.modifiedFileCount ?? 0))
        }}
        files
      </v-chip>
      <v-chip size="small" class="ma-1" color="green" variant="outlined">
        <v-icon start size="small">mdi-harddisk</v-icon>
        {{ filesize((progress?.backupProgress?.newFileSize ?? 0n) + (progress?.backupProgress?.modifiedFileSize ?? 0n))
        }}
      </v-chip>
      <v-chip size="small" class="ma-1" color="orange" variant="outlined">
        <v-icon start size="small">mdi-zip-box</v-icon>
        {{ filesize((progress?.backupProgress?.newCompressedFileSize ?? 0n) +
          (progress?.backupProgress?.modifiedCompressedFileSize ?? 0n)) }}
      </v-chip>
      <v-chip v-if="progress?.backupProgress?.errorCount ?? 0 > 0" size="small" class="ma-1" color="error"
        variant="outlined">
        <v-icon start size="small">mdi-alert</v-icon>
        {{ toNumber(progress?.backupProgress?.errorCount) }} errors
      </v-chip>
      <v-chip v-if="progress?.backupErrorState" size="small" class="ma-1" color="error" variant="flat">
        <v-icon start size="small">mdi-alert-circle</v-icon>
        FAILED
      </v-chip>
    </template>

    <template #details>
      <v-container>
        <!-- Pre-Commands -->
        <v-row v-if="progress?.preCommandStates?.length" class="mb-4">
          <v-col cols="12">
            <h4 class="text-subtitle-2 mb-2">
              <v-icon size="small" class="mr-2">mdi-play-circle</v-icon>
              Pre-Commands
            </h4>
            <div v-for="(preCmd, index) in progress.preCommandStates" :key="index" class="mb-2">
              <v-chip :color="commandStateColor(preCmd.executionState)" variant="outlined" size="small">
                <v-icon start size="small">{{ commandStateIcon(preCmd.executionState) }}</v-icon>
                {{ preCmd.command.command }}
              </v-chip>
            </div>
          </v-col>
        </v-row>

        <!-- Shares Progress -->
        <v-row v-if="progress?.shareStates?.length" class="mb-4">
          <v-col cols="12">
            <h4 class="text-subtitle-2 mb-3">
              <v-icon size="small" class="mr-2">mdi-folder-multiple</v-icon>
              Shares Progress
            </h4>
            <div v-for="(share, index) in progress.shareStates" :key="index" class="mb-4">
              <div class="d-flex align-center mb-2">
                <v-chip :color="shareStateColor(share.executionState)" variant="outlined" class="mr-2">
                  <v-icon start size="small">{{ shareStateIcon(share.executionState) }}</v-icon>
                  {{ share.share }}
                </v-chip>
                <span class="text-caption text-secondary ml-2">
                  {{ shareStateText(share.executionState) }}
                </span>
              </div>

              <!-- Progress bar: Only show for InProgress shares with backup progression -->
              <v-progress-linear
                v-if="share.executionState === ShareExecutionState.InProgress && share.backupProgression"
                :color="shareStateColor(share.executionState)" :model-value="share.backupProgression.percent"
                height="20" class="mb-2">
                <small>{{ toPercent(share.backupProgression.percent) }}</small>
              </v-progress-linear>

              <!-- FileList phase: Show file scanning progress (uses fileListProgression) -->
              <div v-if="share.executionState === ShareExecutionState.FileList && share.fileListProgression"
                class="text-caption text-secondary">
                <!-- Indeterminate progress bar for file scanning -->
                <v-progress-linear color="orange" indeterminate height="20" class="mb-2">
                  <span class="text-orange">Scanning files...</span>
                </v-progress-linear>
                <span class="mr-4">
                  <v-icon size="small">mdi-harddisk</v-icon>
                  {{ filesize(share.fileListProgression.fileSize) }} of new file
                </span>
                <span class="mr-4">
                  <v-icon size="small">mdi-plus-circle</v-icon>
                  {{ toNumber(share.fileListProgression.newFileCount) }} new file
                </span>
                <span class="mr-4">
                  <v-icon size="small">mdi-pencil</v-icon>
                  {{ toNumber(share.fileListProgression.modifiedFileCount) }} modified file
                </span>
                <span class="mr-4">
                  <v-icon size="small">mdi-file</v-icon>
                  {{ filesize(share.fileListProgression.modifiedFileSize) }} modified file
                </span>
                <span v-if="share.fileListProgression.removedFileCount > 0" class="mr-4">
                  <v-icon size="small">mdi-minus-circle</v-icon>
                  {{ toNumber(share.fileListProgression.removedFileCount) }} removed
                </span>
              </div>

              <!-- InProgress phase: Show backup progression (uses backupProgression) -->
              <div
                v-else-if="[ShareExecutionState.InProgress, ShareExecutionState.InProgress].includes(share.executionState) && share.backupProgression"
                class="text-caption text-secondary">
                <span class="mr-4">
                  <v-icon size="small">mdi-file-plus</v-icon>
                  {{ toNumber(share.backupProgression.fileCount) }} files
                </span>
                <span class="mr-4">
                  <v-icon size="small">mdi-harddisk</v-icon>
                  {{ filesize(share.backupProgression.fileSize) }}
                </span>
                <span class="mr-4">
                  <v-icon size="small">mdi-plus-circle</v-icon>
                  {{ toNumber(share.backupProgression.newFileCount) }} new
                </span>
                <span class="mr-4">
                  <v-icon size="small">mdi-file</v-icon>
                  {{ filesize(share.backupProgression.newFileSize) }} new
                </span>
                <span class="mr-4">
                  <v-icon size="small">mdi-pencil</v-icon>
                  {{ toNumber(share.backupProgression.modifiedFileCount) }} modified
                </span>
                <span class="mr-4">
                  <v-icon size="small">mdi-file-edit</v-icon>
                  {{ filesize(share.backupProgression.modifiedFileSize) }} modified
                </span>
                <span v-if="share.backupProgression.speed > 0" class="mr-4">
                  <v-icon size="small">mdi-speedometer</v-icon>
                  {{ filesize(share.backupProgression.speed) }}/s
                </span>
                <span v-if="share.backupProgression.errorCount > 0" class="mr-4 text-error">
                  <v-icon size="small">mdi-alert</v-icon>
                  {{ share.backupProgression.errorCount }} errors
                </span>
              </div>

              <!-- Failed phase: Show error information (uses backupProgression) -->
              <div v-else-if="share.executionState === ShareExecutionState.Failed" class="text-caption text-secondary">
                <div class="mb-2">
                  <v-icon size="small" class="mr-1 text-error">mdi-alert-circle</v-icon>
                  <span class="text-error">Failed</span>
                </div>
              </div>

              <!-- Waiting phase: Show estimation (uses backupProgression if available, otherwise fileListProgression) -->
              <div v-else-if="share.executionState === ShareExecutionState.Waiting" class="text-caption text-secondary">
                <!-- Indeterminate progress bar for waiting state -->
                <v-progress-linear color="grey" indeterminate height="20" class="mb-2">
                  <span>Waiting to start...</span>
                </v-progress-linear>
              </div>
            </div>
          </v-col>
        </v-row>

        <!-- Post-Commands -->
        <v-row v-if="progress?.postCommandStates?.length">
          <v-col cols="12">
            <h4 class="text-subtitle-2 mb-2">
              <v-icon size="small" class="mr-2">mdi-stop-circle</v-icon>
              Post-Commands
            </h4>
            <div v-for="(postCmd, index) in progress.postCommandStates" :key="index" class="mb-2">
              <v-chip :color="commandStateColor(postCmd.executionState)" variant="outlined" size="small">
                <v-icon start size="small">{{ commandStateIcon(postCmd.executionState) }}</v-icon>
                {{ postCmd.command.command }}
              </v-chip>
            </div>
          </v-col>
        </v-row>
      </v-container>
    </template>
  </AbstractTaskCard>
</template>

<script lang="ts" setup>
import { computed } from 'vue';
import type { BackupTaskStateFragment, JobBackupDataFragment } from '@/generated/graphql';
import { BackupExecutionState, ShareExecutionState, ExecuteCommandExecutionState, BackupErrorState } from '@/generated/graphql';
import { toDateTime, toPercent, toNumber } from '@/components/hosts/hosts.utils';
import filesize from '@/utils/filesize';
import AbstractTaskCard from './AbstractTaskCard.vue';

const { data, progress } = defineProps<{
  data: JobBackupDataFragment;
  progress: BackupTaskStateFragment | undefined | null;
}>();

const title = computed(() => `Backup ${data.host}${data.startDate ? ' - ' + toDateTime(data.startDate) : ''}`);

const progressMessage = computed(() => {
  if ((progress?.backupProgress?.speed ?? 0) > 0) {
    return `${filesize(progress?.backupProgress?.speed ?? 0)}/s`;
  }
  return '';
});

const errorMessage = computed(() => {
  const errors = [];
  if (progress?.backupErrorState) {
    switch (progress.backupErrorState) {
      case BackupErrorState.AuthenticationError:
        errors.push('Authentication failed');
        break;
      case BackupErrorState.InitializationError:
        errors.push('Initialization failed');
        break;
      case BackupErrorState.CommandExecutionError:
        errors.push('Command execution failed');
        break;
      case BackupErrorState.BackupError:
        errors.push('Backup operation failed');
        break;
      case BackupErrorState.CompactError:
        errors.push('Compaction failed');
        break;
      case BackupErrorState.CountReferencesError:
        errors.push('Reference counting failed');
        break;
      case BackupErrorState.AddReferencesToPoolError:
        errors.push('Failed to add references to pool');
        break;
      case BackupErrorState.Unknown:
      default:
        errors.push('Unknown error occurred');
    }
  }
  if (progress?.backupErrorMessage) {
    errors.push(progress.backupErrorMessage);
  }
  return errors.join(' - ');
});

const executionStateMessage = computed(() => {
  switch (progress?.backupExecutionState) {
    case BackupExecutionState.DownloadChunks:
      return 'Downloading chunks...';
    case BackupExecutionState.DownloadFileList:
      return 'Downloading file list...';
    case BackupExecutionState.Initialization:
      return 'Initializing...';
    case BackupExecutionState.Authenticate:
      return 'Authenticating...';
    case BackupExecutionState.PreCommands:
      return 'Running pre-commands...';
    case BackupExecutionState.PostCommands:
      return 'Running post-commands...';
    case BackupExecutionState.Compact:
      return 'Compacting...';
    case BackupExecutionState.CountReferences:
      return 'Counting references...';
    case BackupExecutionState.AddReferencesToPool:
      return 'Adding references to pool...';
    case BackupExecutionState.Completed:
      return 'Completed';
    case BackupExecutionState.Waiting:
      return 'Waiting...';
    default:
      return 'In progress...';
  }
});

function commandStateColor(state: ExecuteCommandExecutionState): string {
  switch (state) {
    case ExecuteCommandExecutionState.Success:
      return 'success';
    case ExecuteCommandExecutionState.Failed:
      return 'error';
    case ExecuteCommandExecutionState.InProgress:
      return 'primary';
    case ExecuteCommandExecutionState.Waiting:
      return 'secondary';
    default:
      return 'grey';
  }
}

function commandStateIcon(state: ExecuteCommandExecutionState): string {
  switch (state) {
    case ExecuteCommandExecutionState.Success:
      return 'mdi-check-circle';
    case ExecuteCommandExecutionState.Failed:
      return 'mdi-alert-circle';
    case ExecuteCommandExecutionState.InProgress:
      return 'mdi-loading';
    case ExecuteCommandExecutionState.Waiting:
      return 'mdi-clock-outline';
    default:
      return 'mdi-help-circle';
  }
}

function shareStateColor(state: ShareExecutionState): string {
  switch (state) {
    case ShareExecutionState.InProgress:
      return 'primary';
    case ShareExecutionState.Waiting:
      return 'grey';
    case ShareExecutionState.Success:
      return 'success';
    case ShareExecutionState.Failed:
      return 'error';
    case ShareExecutionState.FileList:
      return 'secondary';
    default:
      return 'grey';
  }
}

function shareStateIcon(state: ShareExecutionState): string {
  switch (state) {
    case ShareExecutionState.InProgress:
      return 'mdi-progress-upload';
    case ShareExecutionState.Waiting:
      return 'mdi-clock-outline';
    case ShareExecutionState.Success:
      return 'mdi-check-circle';
    case ShareExecutionState.Failed:
      return 'mdi-alert-circle';
    case ShareExecutionState.FileList:
      return 'mdi-file-search';
    default:
      return 'mdi-folder';
  }
}

function shareStateText(state: ShareExecutionState): string {
  switch (state) {
    case ShareExecutionState.InProgress:
      return 'Backup in progress';
    case ShareExecutionState.Waiting:
      return 'Waiting to start';
    case ShareExecutionState.Success:
      return 'Completed successfully';
    case ShareExecutionState.Failed:
      return 'Failed';
    case ShareExecutionState.FileList:
      return 'Scanning files...';
    default:
      return 'Unknown state';
  }
}
</script>
