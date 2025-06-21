<template>
  <template v-if="job.name === 'backup' && backupData">
    <TaskCardBackup :data="backupData" :progress="backupProgress">
    </TaskCardBackup>
  </template>
  <template v-else-if="job.name === 'restore' && restoreData">
    <TaskCardRestore :data="restoreData" :progress="restoreProgress">
    </TaskCardRestore>
  </template>
  <template v-else-if="job.name === 'remove_backup' && removeData">
    <TaskCardRemove :data="removeData" :progress="removeProgress">
    </TaskCardRemove>
  </template>
  <template v-else-if="job.name === 'fsck' && fsckData">
    <TaskCardFsck :data="fsckData" :progress="fsckProgress">
    </TaskCardFsck>
  </template>
  <template v-else-if="job.name === 'cleanup_refcnt' && cleanupData">
    <TaskCardCleanup :data="cleanupData" :progress="cleanupProgress">
    </TaskCardCleanup>
  </template>
</template>

<script setup lang="ts">
import { BackupTaskStateFragmentDoc, CleanerTaskStateFragmentDoc, FsckTaskStateFragmentDoc, JobBackupDataFragmentDoc, JobCleanupDataFragmentDoc, JobFsckDataFragmentDoc, JobRemoveDataFragmentDoc, JobRestoreDataFragmentDoc, RemoveTaskStateFragmentDoc, RestoreTaskStateFragmentDoc, type JobFragment } from '@/generated/graphql';
import { useFragment } from '@/generated';
import { computed } from 'vue';
import TaskCardBackup from './TaskCardBackup.vue';
import TaskCardRestore from './TaskCardRestore.vue';
import TaskCardRemove from './TaskCardRemove.vue';
import TaskCardFsck from './TaskCardFsck.vue';
import TaskCardCleanup from './TaskCardCleanup.vue';

const { job } = defineProps<{
  job: JobFragment
}>();

const backupData = computed(() => {
  if (!job.data || job.data.__typename !== 'JobBackupData') return null;
  return useFragment(JobBackupDataFragmentDoc, job.data);
});
const backupProgress = computed(() => {
  if (!job.progression || job.progression.__typename !== 'BackupTaskState') return null;
  return useFragment(BackupTaskStateFragmentDoc, job.progression);
});

const restoreData = computed(() => {
  if (!job.data || job.data.__typename !== 'JobRestoreData') return null;
  return useFragment(JobRestoreDataFragmentDoc, job.data);
});
const restoreProgress = computed(() => {
  if (!job.progression || job.progression.__typename !== 'RestoreTaskState') return null;
  return useFragment(RestoreTaskStateFragmentDoc, job.progression);
});

const removeData = computed(() => {
  if (!job.data || job.data.__typename !== 'JobRemoveData') return null;
  return useFragment(JobRemoveDataFragmentDoc, job.data);
});
const removeProgress = computed(() => {
  if (!job.progression || job.progression.__typename !== 'RemoveTaskState') return null;
  return useFragment(RemoveTaskStateFragmentDoc, job.progression);
});

const fsckData = computed(() => {
  if (!job.data || job.data.__typename !== 'JobFsckData') return null;
  return useFragment(JobFsckDataFragmentDoc, job.data);
});
const fsckProgress = computed(() => {
  if (!job.progression || job.progression.__typename !== 'FsckTaskState') return null;
  return useFragment(FsckTaskStateFragmentDoc, job.progression);
});

const cleanupData = computed(() => {
  if (!job.data || job.data.__typename !== 'JobCleanupData') return null;
  return useFragment(JobCleanupDataFragmentDoc, job.data);
});
const cleanupProgress = computed(() => {
  if (!job.progression || job.progression.__typename !== 'CleanerTaskState') return null;
  return useFragment(CleanerTaskStateFragmentDoc, job.progression);
});

</script>
