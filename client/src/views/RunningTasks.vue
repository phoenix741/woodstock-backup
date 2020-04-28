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
import { Component, Vue } from 'vue-property-decorator';
import Task from '../components/Task.vue';
import runningTasks from './RunningTasks.graphql';
import runningTasksSub from './RunningTasksSubscription.graphql';
import { RunningTasksQuery, FragmentJobFragment, RunningTasksSubSubscription } from '../generated/graphql';

type QueueQuery = RunningTasksQuery['queue']['all'];
type RunningTaskQueue = {
  [key in string]: QueueQuery;
};

@Component({
  components: { Task },
  apollo: {
    runningTasks: {
      query: runningTasks,
      update: ({ queue }: RunningTasksQuery) => queue.all,
      subscribeToMore: {
        document: runningTasksSub,
        updateQuery: (
          previous: RunningTasksQuery,
          { subscriptionData }: { subscriptionData: { data: RunningTasksSubSubscription } },
        ) => {
          const index = previous.queue.all.findIndex(task => task.id === subscriptionData.data.jobUpdated.id);
          if (index >= 0) {
            return;
          }

          const result = {
            ...previous,
            queue: { ...previous.queue, all: [...previous.queue.all, subscriptionData.data.jobUpdated] },
          };
          return result;
        },
      },
    },
  },
})
export default class RunningTasks extends Vue {
  runningTasks: QueueQuery = [];
  tab = 0;

  get runningTasksNotEmpty() {
    return this.runningTasks.reduce((acc: RunningTaskQueue, job: FragmentJobFragment) => {
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
