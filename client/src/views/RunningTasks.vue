<template>
  <v-tabs vertical dark v-model="tab">
    <v-tab class="pa-7" v-for="(tasks, queueType) in runningTasksNotEmpty" :key="queueType">
      <v-badge :color="getColorOf(queueType)" :content="tasks.length">
        {{ queueType }}
      </v-badge>
    </v-tab>

    <v-tabs-items v-model="tab">
      <v-tab-item v-for="(tasks, queueType) in runningTasksNotEmpty" :key="queueType">
        <v-subheader>{{ queueType | capitalize }}</v-subheader>

        <v-expansion-panels>
          <Task v-for="task in tasks || []" :key="task.id" :task="task"></Task>
        </v-expansion-panels>
      </v-tab-item>
    </v-tabs-items>
  </v-tabs>
</template>

<script lang="ts">
import { mixins } from 'vue-class-component';
import { Component } from 'vue-property-decorator';
import Task from '../components/Task.vue';
import runningTasks from './RunningTasks.graphql';
import runningTasksSub from './RunningTasksSubscription.graphql';
import { RunningTasksQuery, RunningTasksSubSubscription } from '../generated/graphql';
import { QueueComponent } from '../components/QueueComponent';

type QueueQuery = RunningTasksQuery['queue']['all'];
type RunningTaskQueue = {
  [key in string]: QueueQuery;
};

@Component({
  components: { Task },
})
export default class RunningTasks extends mixins(
  QueueComponent<RunningTasksQuery, RunningTasksSubSubscription>(runningTasks, runningTasksSub, 'all'),
) {
  tab = 0;

  get runningTasksNotEmpty() {
    return this.runningTasks.reduce((acc, job) => {
      if (job.state) {
        acc[job.state] = acc[job.state] || [];
        acc[job.state].push(job);
      }
      return acc;
    }, {} as RunningTaskQueue);
  }

  getColorOf(type: string) {
    switch (type) {
      case 'active':
        return 'primary';
      case 'completed':
        return 'green';
      case 'failed':
        return 'red';
      case 'waiting':
      case 'delayed':
        return 'yellow';
      default:
        return 'secondary';
    }
  }
}
</script>

<style scoped>
.v-tabs {
  min-height: 93vh;
}
</style>
