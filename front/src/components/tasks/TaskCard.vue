<template>
  <v-sheet rounded="lg">
    <v-expansion-panels>
      <v-expansion-panel>
        <v-expansion-panel-title>
          <v-container>
            <v-row no-gutters>
              <v-col cols="8">
                <span class="font-weight-light">
                  <v-icon small>mdi-cloud-download</v-icon>
                  {{ task.jobType }}
                  {{ task.hostname }}
                  {{ task.startDate && ' - ' + toDateTime(task.startDate) }}
                </span>
              </v-col>
              <v-col cols="4" v-if="message">
                <div class="text-right font-weight-light">{{ message }}</div>
              </v-col>
            </v-row>
            <v-row class="pt-5" no-gutters v-if="task.state === QueueTaskState.Running">
              <v-col cols="12">
                <v-progress-linear color="primary" striped :model-value="progress" :indeterminate="false" height="25">
                  <strong
                    >{{ toPercent(progress) }}
                    <template v-if="task.progression?.speed"
                      >({{ filesize(task.progression.speed) }}/s)</template
                    ></strong
                  >
                </v-progress-linear>
              </v-col>
            </v-row>
            <v-row class="pt-5" no-gutters v-if="task.state === QueueTaskState.Success">
              <v-col cols="12">
                <v-progress-linear color="success" striped :model-value="100" :indeterminate="false" height="25">
                </v-progress-linear>
              </v-col>
            </v-row>
            <v-row class="pt-5" no-gutters v-if="task.state === QueueTaskState.Failed">
              <v-col cols="12">
                <v-progress-linear color="error" striped :model-value="progress" :indeterminate="false" height="25">
                  <strong>{{ task.failedReason }}</strong>
                </v-progress-linear>
              </v-col>
            </v-row>
          </v-container>
        </v-expansion-panel-title>
        <v-expansion-panel-text v-if="task.details.length">
          <v-list>
            <v-list-item
              v-for="(subtask, index) in task.details"
              :key="task.jobId + '-' + index"
              :active="subtask.state === QueueTaskState.Running"
            >
              <v-list-item-title>{{ subtask.title }}</v-list-item-title>
              <v-list-item-subtitle
                >{{ subtask.description }} {{ toSubtaskProgress(subtask.progression) }}</v-list-item-subtitle
              >
              <template v-slot:append>
                <v-icon v-if="subtask.state === QueueTaskState.Success" class="text-green"
                  >mdi-checkbox-marked-circle-outline</v-icon
                >
                <v-icon v-else-if="subtask.state === QueueTaskState.Waiting">mdi-clock-outline</v-icon>
                <v-progress-circular
                  v-else-if="subtask.state === QueueTaskState.Running && taskRunning"
                  :indeterminate="!subtask.progression?.progressMax"
                  :model-value="
                    Number(
                      ((subtask.progression?.progressCurrent ?? 0n) * 100n) / (subtask.progression?.progressMax || 1n),
                    )
                  "
                  color="primary"
                ></v-progress-circular>
                <v-icon
                  v-else-if="
                    subtask.state === QueueTaskState.Failed ||
                    (subtask.state === QueueTaskState.Running && !taskRunning)
                  "
                  class="text-red"
                  >mdi-alert-circle-outline</v-icon
                >
                <v-icon v-else-if="subtask.state === QueueTaskState.Aborted" class="text-grey">mdi-cancel</v-icon>
              </template>
            </v-list-item>
          </v-list>
        </v-expansion-panel-text>
      </v-expansion-panel>
    </v-expansion-panels>
  </v-sheet>
</template>

<script setup lang="ts">
import { toDateTime, toNumber, toPercent } from '@/components/hosts/hosts.utils';
import { QueueTaskState } from '@/generated/graphql';
import { JobTaskGui, ProgressTaskDetailGui } from '@/utils/tasks.interface';
import filesize from '@/utils/filesize';
import { computed, defineProps } from 'vue';

const props = defineProps<{
  task: JobTaskGui;
}>();

const message = computed(() => {
  if (props.task.progression?.newFileCount) {
    return `${props.task.progression?.newFileCount} ${
      props.task.progression?.fileCount ? '/' + props.task.progression?.fileCount : ''
    } files`;
  }
  return undefined;
});

const progress = computed(() => {
  if (!props.task.progression?.progressCurrent || !props.task.progression?.progressMax) {
    return undefined;
  }

  return Number((props.task.progression?.progressCurrent * 100n) / props.task.progression?.progressMax);
});

const taskRunning = computed(() => {
  return props.task.state === QueueTaskState.Running;
});

function toSubtaskProgress(progress?: ProgressTaskDetailGui) {
  const message = [];
  if (progress?.newFileCount) {
    message.push(`${toNumber(progress?.newFileCount)} new files`);
  }
  if (progress?.fileCount) {
    message.push(`${toNumber(progress?.fileCount)} files`);
  }
  if (progress?.errorCount) {
    message.push(`${toNumber(progress?.errorCount)} errors`);
  }
  if (progress?.newFileSize) {
    message.push(`${filesize(progress?.newFileSize)}`);
  }

  return message.join(' - ');
}
</script>
