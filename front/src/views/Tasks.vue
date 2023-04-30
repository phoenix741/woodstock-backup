<template>
  <v-container>
    <v-row>
      <v-col cols="2" v-if="!isMenuFetching">
        <TasksMenu :stats="menuData"></TasksMenu>
      </v-col>

      <v-col v-if="!isFetching">
        <v-row v-for="task in tasks" :key="task.jobId">
          <v-col>
            <TaskCard :task="task"></TaskCard>
          </v-col>
        </v-row>
      </v-col>

      <v-col v-else class="text-center">
        <v-progress-circular indeterminate></v-progress-circular>
      </v-col>
    </v-row>
  </v-container>
</template>

<script lang="ts" setup>
import TaskCard from '@/components/tasks/TaskCard.vue';
import TasksMenu from '@/components/tasks/TasksMenu.vue';
import { useQueueRealtimeStats } from '@/utils/stats';
import { useTasks } from '@/utils/tasks';
import { computed, ref, toRefs } from 'vue';

const props = defineProps<{
  taskFilter: string;
}>();

const taskFilterRef = toRefs(props).taskFilter;

const { queueStats: menuData, isFetching: isMenuFetching, refetch } = useQueueRealtimeStats();

const { tasks, isFetching } = useTasks(
  computed(() => [taskFilterRef.value]),
  ref(undefined),
  refetch,
);
</script>
