<template>
  <v-expansion-panel>
    <v-expansion-panel-header>
      <v-container>
        <v-row no-gutters>
          <v-col cols="8">
            <span class="font-weight-light">
              <v-icon small>mdi-cloud-download</v-icon> {{ task.data.host }}
              <template v-if="task.data.startDate">- {{ task.data.startDate | date }}</template>
            </span>
          </v-col>
          <v-col cols="4" v-if="taskRunning">
            <div class="text-right font-weight-light">{{ progressText }}</div>
          </v-col>
        </v-row>
        <v-row class="pt-5" no-gutters v-if="taskRunning || taskFailed">
          <v-col cols="12" v-if="taskRunning">
            <v-progress-linear
              color="primary"
              striped
              :value="(task.data.progression || {}).percent"
              :indeterminate="!(task.data.progression || {}).percent"
              height="25"
            >
              <template v-slot="{ value }">
                <strong
                  >{{ value | formatPercent }} ({{ (task.data.progression || {}).speed || 0 | filesize }}/s)
                </strong>
              </template>
            </v-progress-linear>
          </v-col>
          <v-col cols="12" v-else-if="taskFailed">
            <v-progress-linear color="red" striped :value="(task.data.progression || {}).percent" height="25">
              <template v-slot="{ value }">
                <strong>{{ value | formatPercent }} ({{ task.failedReason }}) </strong>
              </template>
            </v-progress-linear>
          </v-col>
        </v-row>
      </v-container>
    </v-expansion-panel-header>
    <v-expansion-panel-content>
      <v-list>
        <template v-for="(subtask, index) in task.data.subtasks || []">
          <v-list-item :key="subtask.context + '-item' + index">
            <v-list-item-content>
              <v-list-item-title>{{ subtask.description }}</v-list-item-title>
            </v-list-item-content>
            <v-list-item-avatar v-if="subtask.state === 'SUCCESS'">
              <v-icon class="green--text">mdi-checkbox-marked-circle-outline</v-icon>
            </v-list-item-avatar>
            <v-list-item-avatar v-else-if="subtask.state === 'WAITING'">
              <v-icon>mdi-clock-outline</v-icon>
            </v-list-item-avatar>
            <v-list-item-avatar v-else-if="subtask.state === 'RUNNING' && taskRunning">
              <v-progress-circular indeterminate color="primary"></v-progress-circular>
            </v-list-item-avatar>
            <v-list-item-avatar v-else-if="subtask.state === 'FAILED' || (subtask.state === 'RUNNING' && !taskRunning)">
              <v-icon class="red--text">mdi-alert-circle-outline</v-icon>
            </v-list-item-avatar>
            <v-list-item-avatar v-else-if="subtask.state === 'ABORTED'">
              <v-icon class="grey--text">mdi-cancel</v-icon>
            </v-list-item-avatar>
          </v-list-item>
          <v-divider
            :key="subtask.context + '-divider' + index"
            v-if="index !== (task.data.subtasks || []).length - 1"
          ></v-divider>
        </template>
      </v-list>
    </v-expansion-panel-content>
  </v-expansion-panel>
</template>

<script lang="ts">
import { Vue, Component, Prop } from 'vue-property-decorator';
import { Job } from '@/generated/graphql';

@Component({})
export default class Task extends Vue {
  @Prop()
  task!: Job;

  get taskRunning() {
    return this.task.state === 'active';
  }

  get taskFailed() {
    return this.task.state === 'failed' || this.task.data.state === 'FAILED';
  }

  get progressText() {
    return `${this.task.data.progression?.newFileCount} ${
      this.task.data.progression?.fileCount ? '/' + this.task.data.progression?.fileCount : ''
    } files`;
  }
}
</script>
