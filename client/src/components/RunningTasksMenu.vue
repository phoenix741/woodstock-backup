<template>
  <v-menu left bottom v-if="runningTasks && runningTasks.length">
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
import { mixins } from 'vue-class-component';
import { Component } from 'vue-property-decorator';
import runningTasks from './RunningTasksMenu.graphql';
import runningTasksSub from './RunningTasksMenuSubscription.graphql';
import { QueueComponent } from './QueueComponent';
import { RunningTasksMenuQuery, RunningTasksSubSubscription } from '../generated/graphql';

@Component({})
export default class RunningTasksMenu extends mixins(
  QueueComponent<RunningTasksMenuQuery, RunningTasksSubSubscription>(runningTasks, runningTasksSub, 'active'),
) {}
</script>
