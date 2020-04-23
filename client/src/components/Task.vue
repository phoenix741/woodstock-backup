<template>
  <v-expansion-panel>
    <v-expansion-panel-header>
      <v-container>
        <v-row no-gutters>
          <v-col cols="8">
            <span class="font-weight-light">
              <v-icon small>mdi-cloud-download</v-icon> {{ task.name }} -
              {{ startDate }} - {{ task.percent }}%
            </span>
          </v-col>
          <v-col cols="4" v-if="task.state == 'RUNNING'">
            <div class="text-right font-weight-light">{{ progressText }}</div>
          </v-col>
        </v-row>
        <v-row class="pt-5" no-gutters>
          <v-col cols="12" v-if="task.state == 'RUNNING'">
            <v-progress-linear
              v-model="task.percent"
              :indeterminate="!task.percent"
            ></v-progress-linear>
          </v-col>
        </v-row>
      </v-container>
    </v-expansion-panel-header>
    <v-expansion-panel-content>
      <v-list>
        <template v-for="(subtask, index) in task.subtasks">
          <v-list-item :key="subtask.name + '-item'">
            <v-list-item-content>
              <v-list-item-title>{{ subtask.name }}</v-list-item-title>
            </v-list-item-content>
            <v-list-item-avatar v-if="subtask.state === 'SUCCESS'">
              <v-icon class="green--text"
                >mdi-checkbox-marked-circle-outline</v-icon
              >
            </v-list-item-avatar>
            <v-list-item-avatar v-else-if="subtask.state === 'WAITING'">
              <v-icon>mdi-clock-outline</v-icon>
            </v-list-item-avatar>
            <v-list-item-avatar v-else-if="subtask.state === 'RUNNING'">
              <v-progress-circular
                indeterminate
                color="primary"
              ></v-progress-circular>
            </v-list-item-avatar>
            <v-list-item-avatar v-else-if="subtask.state === 'FAILED'">
              <v-icon class="red--text">mdi-alert-circle-outline</v-icon>
            </v-list-item-avatar>
            <v-list-item-avatar v-else-if="subtask.state === 'ABORTED'">
              <v-icon class="grey--text">mdi-cancel</v-icon>
            </v-list-item-avatar>
          </v-list-item>
          <v-divider
            :key="subtask.name + '-divider'"
            v-if="index !== task.subtasks.length - 1"
          ></v-divider>
        </template>
      </v-list>
    </v-expansion-panel-content>
  </v-expansion-panel>
</template>

<script>
import { humanize } from "humanize";
import { Vue, Component } from "vue-property-decorator";

@Component({})
export default class Task extends Vue {
  task = {
    name: "pc-ulrich",
    speed: 100000,
    newFileCount: 123123,
    fileCount: 1545343,
    state: "RUNNING",
    percent: 35,
    subtasks: [
      { name: "task 1", state: "RUNNING" },
      { name: "task 2", state: "WAITING" },
      { name: "task 3", state: "FAILED" }
    ],
    startDate: new Date()
  };

  get progressText() {
    return `${humanize.filesize(this.task.speed)}/s - ${
      this.task.newFileCount
    } ${this.task.fileCount ? "/" + this.task.fileCount : ""}`;
  }

  get startDate() {
    return humanize.naturalDay(this.task.startDate, "m/j/Y H:i");
  }
}
</script>
