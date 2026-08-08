<template>
  <v-container fluid>
    <!-- Filtres par type + compteurs de statut -->
    <v-row class="mb-2" align="center">
      <v-col cols="12">
        <div class="d-flex flex-wrap align-center ga-2">
          <!-- Filtre par type de job -->
          <v-chip
            :variant="selectedKind === null ? 'flat' : 'tonal'"
            :color="selectedKind === null ? 'primary' : 'default'"
            @click="selectedKind = null"
          >
            All
          </v-chip>
          <v-chip
            v-for="kind in availableKinds"
            :key="kind.value"
            :variant="selectedKind === kind.value ? 'flat' : 'tonal'"
            :color="selectedKind === kind.value ? 'primary' : 'default'"
            :prepend-icon="kind.icon"
            @click="selectedKind = selectedKind === kind.value ? null : kind.value"
          >
            {{ kind.label }}
          </v-chip>

          <v-spacer />

          <!-- Compteurs (informatifs uniquement) -->
          <v-chip color="info" variant="outlined" size="small">
            <v-icon start size="small">mdi-play-circle</v-icon>
            {{ toNumber(runningTasks.length) }}
          </v-chip>
          <v-chip color="warning" variant="outlined" size="small">
            <v-icon start size="small">mdi-clock-outline</v-icon>
            {{ toNumber(pendingTasks.length) }}
          </v-chip>
          <v-chip color="success" variant="outlined" size="small">
            <v-icon start size="small">mdi-check-circle</v-icon>
            {{ toNumber(completedTasks.length) }}
          </v-chip>
          <v-chip color="error" variant="outlined" size="small">
            <v-icon start size="small">mdi-alert-circle</v-icon>
            {{ toNumber(failedTasks.length) }}
          </v-chip>
        </div>
      </v-col>
    </v-row>

    <!-- Chargement initial -->
    <v-row v-if="isLoading" justify="center" class="mt-6">
      <v-col cols="auto">
        <v-progress-circular indeterminate />
      </v-col>
    </v-row>

    <template v-else>
      <!-- ── EN COURS ── -->
      <template v-if="runningFiltered.length > 0">
        <v-row class="mt-2">
          <v-col cols="12">
            <div class="d-flex align-center mb-2">
              <v-icon color="info" class="mr-2">mdi-play-circle</v-icon>
              <span class="text-subtitle-1 font-weight-bold">Running</span>
              <v-chip size="x-small" color="info" variant="tonal" class="ml-2">{{
                toNumber(runningFiltered.length)
              }}</v-chip>
            </div>
          </v-col>
        </v-row>
        <v-row v-for="task in runningFiltered" :key="task.jobId ?? 'unknown'">
          <v-col>
            <TaskCard :job="task" :expanded="true" />
          </v-col>
        </v-row>
      </template>

      <!-- ── EN ATTENTE ── -->
      <template v-if="pendingFiltered.length > 0">
        <v-row :class="runningFiltered.length > 0 ? 'mt-6' : 'mt-2'">
          <v-col cols="12">
            <div class="d-flex align-center mb-2">
              <v-icon color="warning" class="mr-2">mdi-clock-outline</v-icon>
              <span class="text-subtitle-1 font-weight-bold">Pending</span>
              <v-chip size="x-small" color="warning" variant="tonal" class="ml-2">{{
                toNumber(pendingFiltered.length)
              }}</v-chip>
            </div>
          </v-col>
        </v-row>
        <v-row v-for="task in pendingFiltered" :key="task.jobId ?? 'unknown'">
          <v-col>
            <TaskCard :job="task" :expanded="false" />
          </v-col>
        </v-row>
      </template>

      <!-- ── HISTORIQUE ── -->
      <!-- Affiché dès qu'il y a des tâches terminées/échouées, même si le filtre courant en masque certaines -->
      <template v-if="historyTotal > 0">
        <v-row :class="runningFiltered.length > 0 || pendingFiltered.length > 0 ? 'mt-6' : 'mt-2'">
          <v-col cols="12">
            <div class="d-flex align-center mb-2">
              <v-icon class="mr-2">mdi-history</v-icon>
              <span class="text-subtitle-1 font-weight-bold">History</span>
              <v-chip size="x-small" variant="tonal" class="ml-2">{{ toNumber(historyFiltered.length) }}</v-chip>
              <v-spacer />
              <!-- Sous-filtre : Terminés / Échoués -->
              <v-btn-toggle v-model="historySubFilter" multiple density="compact">
                <v-btn value="completed" size="small" color="success" variant="text">
                  <v-icon start size="small">mdi-check-circle</v-icon>
                  Completed
                </v-btn>
                <v-btn value="failed" size="small" color="error" variant="text">
                  <v-icon start size="small">mdi-alert-circle</v-icon>
                  Failed
                </v-btn>
              </v-btn-toggle>
            </div>
          </v-col>
        </v-row>
        <v-row v-for="task in historyFiltered" :key="task.jobId ?? 'unknown'">
          <v-col>
            <TaskCard :job="task" :expanded="false" />
          </v-col>
        </v-row>
        <!-- Aucun résultat après application des filtres, mais des tâches existent -->
        <v-row v-if="historyFiltered.length === 0">
          <v-col class="text-center text-grey text-body-2 py-4"> No tasks match the selected filters. </v-col>
        </v-row>
      </template>

      <!-- ── État vide ── -->
      <template v-if="isEmpty">
        <v-row class="mt-10" justify="center">
          <v-col cols="auto" class="text-center">
            <v-icon size="72" color="grey-lighten-1">mdi-check-all</v-icon>
            <div class="text-body-1 text-grey mt-2">No tasks</div>
          </v-col>
        </v-row>
      </template>
    </template>
  </v-container>
</template>

<script lang="ts" setup>
import TaskCard from '@/components/tasks/TaskCard.vue';
import { JobKind, JobStatus } from '@/generated/graphql';
import { toNumber } from '@/components/hosts/hosts.utils';
import { useQueueRealtimeStats } from '@/utils/stats';
import { useTasks } from '@/utils/tasks';
import { computed, ref } from 'vue';

const { refetch } = useQueueRealtimeStats();

// Single query for all statuses, client-side filtering
const { tasks: allTasks, isFetching } = useTasks(ref(undefined), ref(undefined), refetch);

// Derived lists by status
const runningTasks = computed(() => allTasks.value.filter((t) => t.status === JobStatus.Started));
const pendingTasks = computed(() => allTasks.value.filter((t) => t.status === JobStatus.Created));
const completedTasks = computed(() => allTasks.value.filter((t) => t.status === JobStatus.Completed));
const failedTasks = computed(() => allTasks.value.filter((t) => t.status === JobStatus.Failed));

// Initial loading
const isLoading = computed(() => isFetching.value);
// Filter by job kind (null = all)
const selectedKind = ref<JobKind | null>(null);

// History sub-filter: both enabled by default
const historySubFilter = ref<string[]>(['completed', 'failed']);

const availableKinds = [
  { value: JobKind.Backup, label: 'Backup', icon: 'mdi-cloud-download' },
  { value: JobKind.Fsck, label: 'Fsck', icon: 'mdi-shield-search' },
  { value: JobKind.Remove, label: 'Remove', icon: 'mdi-delete' },
  { value: JobKind.Restore, label: 'Restore', icon: 'mdi-restore' },
  { value: JobKind.CleanupRefcnt, label: 'Cleanup', icon: 'mdi-broom' },
  { value: JobKind.Archive, label: 'Archive', icon: 'mdi-archive' },
];

function filterByKind<T extends { kind?: string | null }>(tasks: T[]): T[] {
  if (selectedKind.value === null) return tasks;
  return tasks.filter((t) => t.kind === selectedKind.value);
}

const runningFiltered = computed(() => filterByKind(runningTasks.value));
const pendingFiltered = computed(() => filterByKind(pendingTasks.value));

const historyAll = computed(() => {
  const all = [];
  if (historySubFilter.value.includes('completed')) all.push(...completedTasks.value);
  if (historySubFilter.value.includes('failed')) all.push(...failedTasks.value);
  return all;
});
const historyFiltered = computed(() => filterByKind(historyAll.value));

// Raw total before sub-filter: determines if the History section is visible
const historyTotal = computed(() => completedTasks.value.length + failedTasks.value.length);

const isEmpty = computed(
  () =>
    !isLoading.value &&
    runningFiltered.value.length === 0 &&
    pendingFiltered.value.length === 0 &&
    historyTotal.value === 0,
);
</script>
