<template>
  <template v-if="job.kind === JobKind.Backup && backupData">
    <TaskCardBackup
      :data="backupData"
      :progress="backupProgress"
      :job-id="job.jobId"
      :job-status="job.status"
      :expanded="expanded"
    />
  </template>
  <template v-else-if="job.kind === JobKind.Restore && restoreData">
    <TaskCardRestore
      :data="restoreData"
      :progress="restoreProgress"
      :job-id="job.jobId"
      :job-status="job.status"
      :expanded="expanded"
    />
  </template>
  <template v-else-if="job.kind === JobKind.Remove && removeData">
    <TaskCardRemove :data="removeData" :progress="removeProgress" :expanded="expanded" />
  </template>
  <template v-else-if="job.kind === JobKind.Fsck && fsckData">
    <TaskCardFsck
      :data="fsckData"
      :progress="fsckProgress"
      :job-id="job.jobId"
      :job-status="job.status"
      :expanded="expanded"
    />
  </template>
  <template v-else-if="job.kind === JobKind.CleanupRefcnt && cleanupData">
    <TaskCardCleanup :data="cleanupData" :progress="cleanupProgress" :expanded="expanded" />
  </template>
  <template v-else-if="job.kind === JobKind.Archive && archiveData">
    <TaskCardArchive
      :data="archiveData"
      :progress="archiveProgress"
      :status="job.status"
      :job-id="job.jobId"
      :failed-reason="job.failedReason"
      :expanded="expanded"
    />
  </template>
</template>

<script setup lang="ts">
import {
  ArchiveTaskStateFragmentDoc,
  BackupTaskStateFragmentDoc,
  CleanerTaskStateFragmentDoc,
  FsckTaskStateFragmentDoc,
  JobArchiveDataFragmentDoc,
  JobBackupDataFragmentDoc,
  JobCleanupDataFragmentDoc,
  JobFsckDataFragmentDoc,
  JobKind,
  JobRemoveDataFragmentDoc,
  JobRestoreDataFragmentDoc,
  RemoveTaskStateFragmentDoc,
  RestoreTaskStateFragmentDoc,
  type JobFragment,
} from '@/generated/graphql';
import { useFragment } from '@/generated';
import { computed } from 'vue';
import TaskCardBackup from './TaskCardBackup.vue';
import TaskCardRestore from './TaskCardRestore.vue';
import TaskCardRemove from './TaskCardRemove.vue';
import TaskCardFsck from './TaskCardFsck.vue';
import TaskCardCleanup from './TaskCardCleanup.vue';
import TaskCardArchive from './TaskCardArchive.vue';

const { job, expanded } = defineProps<{
  job: JobFragment;
  expanded?: boolean;
}>();

const backupData = computed(() => {
  if (!job.data || job.data.__typename !== 'JobBackupData') return null;
  return useFragment(JobBackupDataFragmentDoc, job.data);
});
const backupProgress = computed(() => {
  if (!job.progress || job.progress.__typename !== 'JobBackupTaskState') return null;
  return useFragment(BackupTaskStateFragmentDoc, job.progress);
});

const restoreData = computed(() => {
  if (!job.data || job.data.__typename !== 'JobRestoreData') return null;
  return useFragment(JobRestoreDataFragmentDoc, job.data);
});
const restoreProgress = computed(() => {
  if (!job.progress || job.progress.__typename !== 'JobRestoreTaskState') return null;
  return useFragment(RestoreTaskStateFragmentDoc, job.progress);
});

const removeData = computed(() => {
  if (!job.data || job.data.__typename !== 'JobRemoveData') return null;
  return useFragment(JobRemoveDataFragmentDoc, job.data);
});
const removeProgress = computed(() => {
  if (!job.progress || job.progress.__typename !== 'JobRemoveState') return null;
  return useFragment(RemoveTaskStateFragmentDoc, job.progress);
});

const fsckData = computed(() => {
  if (!job.data || job.data.__typename !== 'JobFsckData') return null;
  return useFragment(JobFsckDataFragmentDoc, job.data);
});
const fsckProgress = computed(() => {
  if (!job.progress || job.progress.__typename !== 'JobFsckTaskState') return null;
  return useFragment(FsckTaskStateFragmentDoc, job.progress);
});

const cleanupData = computed(() => {
  if (!job.data || job.data.__typename !== 'JobCleanupData') return null;
  return useFragment(JobCleanupDataFragmentDoc, job.data);
});
const cleanupProgress = computed(() => {
  if (!job.progress || job.progress.__typename !== 'JobCleanerTaskState') return null;
  return useFragment(CleanerTaskStateFragmentDoc, job.progress);
});

const archiveData = computed(() => {
  if (!job.data || job.data.__typename !== 'JobArchiveData') return null;
  return useFragment(JobArchiveDataFragmentDoc, job.data);
});
const archiveProgress = computed(() => {
  if (!job.progress || job.progress.__typename !== 'JobArchiveTaskState') return null;
  return useFragment(ArchiveTaskStateFragmentDoc, job.progress);
});
</script>
