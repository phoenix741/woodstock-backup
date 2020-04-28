<template>
  <v-menu left bottom>
    <template v-slot:activator="{ on }">
      <v-btn icon v-on="on">
        <v-badge color="red" :content="runningTasks.length" overlap>
          <v-icon>mdi-animation</v-icon>
        </v-badge>
      </v-btn>
    </template>

    <v-card>
      <v-list>
        <v-list-item>
          <v-list-item-content>
            <v-list-item-title>You have {{ runningTasks.length }} tasks running</v-list-item-title>
          </v-list-item-content>
          <v-list-item-action>
            <v-btn rounded color="primary" dark to="/tasks">View all</v-btn>
          </v-list-item-action>
        </v-list-item>
      </v-list>

      <v-divider></v-divider>

      <v-list>
        <v-list-item v-for="task in runningTasks" :key="task.id">
          <v-list-item-avatar>
            <v-progress-circular
              :value="(task.data.progression || {}).percent"
              width="3"
              color="primary"
            ></v-progress-circular>
          </v-list-item-avatar>

          <v-list-item-content>
            <v-list-item-title>{{ task.data.host }}</v-list-item-title>
            <v-list-item-subtitle>{{ (task.data.progression || {}).fileCount }} files</v-list-item-subtitle>
          </v-list-item-content>
        </v-list-item>
      </v-list>
    </v-card>
  </v-menu>
</template>

<script lang="ts">
import { Component, Vue } from 'vue-property-decorator';
import Task from '../components/Task.vue';
import runningTasks from './RunningTasksMenu.graphql';
import runningTasksSub from './RunningTasksMenuSubscription.graphql';
import { RunningTasksMenuQuery, RunningTasksSubSubscription } from '../generated/graphql';

type QueueQuery = RunningTasksMenuQuery['queue']['active'];
type RunningTaskQueue = {
  [key in string]: QueueQuery;
};

@Component({
  components: { Task },
  apollo: {
    runningTasks: {
      query: runningTasks,
      update: ({ queue }: RunningTasksMenuQuery) => queue.active,
      subscribeToMore: {
        document: runningTasksSub,
        updateQuery: (
          previous: RunningTasksMenuQuery,
          { subscriptionData }: { subscriptionData: { data: RunningTasksSubSubscription } },
        ) => {
          const index = previous.queue.active.findIndex(task => task.id === subscriptionData.data.jobUpdated.id);
          if (index >= 0) {
            return;
          }

          const result = {
            ...previous,
            queue: { ...previous.queue, active: [...previous.queue.active, subscriptionData.data.jobUpdated] },
          };
          return result;
        },
      },
    },
  },
})
export default class RunningTasksMenu extends Vue {
  runningTasks: QueueQuery = [];
}
</script>
