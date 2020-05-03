<template>
  <v-list-group prepend-icon="mdi-animation" to="/tasks" v-if="runningTasks.length">
    <template v-slot:activator>
      <v-list-item-title>Running Tasks</v-list-item-title>
    </template>

    <v-list-item :to="'/tasks/' + queueType" v-for="(tasks, queueType) in runningTasksNotEmpty" :key="queueType">
      <v-list-item-title>
        <v-badge :color="getColorOf(queueType)" :content="tasks.length" inline>{{ queueType | capitalize }}</v-badge>
      </v-list-item-title>
    </v-list-item>
  </v-list-group>
</template>

<script lang="ts">
import { mixins } from 'vue-class-component';
import { Component } from 'vue-property-decorator';
import runningTasks from './NavigationBarTasks.graphql';
import runningTasksSub from './NavigationBarTasksJobUpdated.graphql';
import { NavigationBarTasksQuery, NavigationBarTasksJobUpdatedSubscription } from '../generated/graphql';
import { QueueComponent } from '../components/QueueComponent';

type QueueQuery = NavigationBarTasksQuery['queue'];
type RunningTaskQueue = {
  [key in string]: QueueQuery;
};

@Component({})
export default class NavigationBarTasks extends mixins(
  QueueComponent<NavigationBarTasksQuery, NavigationBarTasksJobUpdatedSubscription>(runningTasks, runningTasksSub),
) {
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
