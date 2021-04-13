<template>
  <div>
    <v-subheader>{{ state | capitalize }}</v-subheader>

    <v-expansion-panels>
      <Task v-for="task in runningTasks || []" :key="task.id" :task="task"></Task>
    </v-expansion-panels>
  </div>
</template>

<script lang="ts">
import { mixins } from 'vue-class-component';
import { Component, Prop } from 'vue-property-decorator';
import Task from '@/components/tasks/Task.vue';
import runningTasks from './QueueTasks.graphql';
import runningTasksSub from './QueueTasksSubscription.graphql';
import { QueueTasksQuery, QueueTasksJobUpdatedSubscription } from '@/generated/graphql';
import { QueueComponent } from '@/components/dashboard/queue/QueueComponent.ts';

@Component({
  components: { Task },
})
export default class QueueTasks extends mixins(
  QueueComponent<QueueTasksQuery, QueueTasksJobUpdatedSubscription>(runningTasks, runningTasksSub),
) {
  @Prop({ required: true })
  state!: string;
}
</script>
