import { FragmentType, useFragment } from '@/generated';
import {
  BackupTaskFragmentDoc,
  JobFragmentDoc,
  JobProgression,
  JobProgressionFragmentDoc,
  ProgressTaskFragmentDoc,
  QueueTaskState,
  QueueTasksJobUpdatedDocument,
  TaskDescriptionFragmentDoc,
  TasksDocument,
} from '@/generated/graphql';
import { UseQueryReturn, useQuery } from '@vue/apollo-composable';
import { Ref, computed } from 'vue';
import { JobTaskGui } from './tasks.interface';

const TASKS_GROUP_NAME_TO_DISPLAY: Record<string, (description?: string | null) => string> = {
  // Task
  init: () => 'Initialisation',
  share: (description) => `Backup of share ${description ?? ''}`,
  'pre-command': () => 'Execution of distant command before backup',
  'post-command': () => 'Execution of distant command after backup',
  end: () => 'Finalisation',

  // Subtask
  connection: () => `Connection to host`,
  'init-directory': () => 'Creation of backup directory',
  refreshcache: () => 'Refresh of client file list',
  filelist: () => 'Get the file list from the client',
  chunks: () => 'Get the chunks from the client',
  compact: () => 'Compact the manifest',
  'close-connection': () => 'Close the client connection',
  'refcnt-host': () => 'Count number of reference of the host',
  'refcnt-pool': () => 'Refresh number of reference of the pool',

  // Remove
  REMOVE_REFCNT_POOL_TASK: () => 'Remove reference of the pool',
  REMOVE_REFCNT_HOST_TASK: () => 'Remove reference of the host',
  REMOVE_BACKUP_TASK: () => 'Remove backup',

  // Refcnt
  PREPARE_CLEAN_UNUSED_POOL_TASK: () => 'Prepare clean unused pool',
  CLEAN_UNUSED_POOL_TASK: () => 'Remove all unused file from the pool',
  ADD_REFCNT_POOL_TASK: () => 'Add reference of the host in the pool',

  // Fsck
  prepare: () => 'Prepare verification',
  refcnt_backup: (description) => `Count number of reference of the backup ${description}`,
  refcnt_host: (description) => `Count number of reference of the host ${description}`,
  refcnt_pool: () => 'Count number of reference of the pool',
  refcnt_unused: () => 'Count number of reference of the unused',

  // Verify checksum
  VERIFY_CHECKSUM: () => 'Verify checksum',
};

const BACKUP_TYPE: Record<string, string> = {
  // host
  backup: 'Backup',
  restore: 'Restore',
  remove_backup: 'Remove',

  // refcnt
  add_backup: 'Update pool (add)',
  //remove_backup: 'Update pool (remove)',
  unused: 'Clean pool',
  fsck: 'Verify pool',
  verify_checksum: 'Verify checksum',
};

function getRunningSubtask(details: FragmentType<typeof TaskDescriptionFragmentDoc>[]) {
  const runningSubtask = details
    .map((subtask) => useFragment(TaskDescriptionFragmentDoc, subtask))
    .find((subtask) => subtask.__typename === 'JobSubTask' && subtask.state === QueueTaskState.Running);

  if (runningSubtask?.__typename === 'JobSubTask') {
    return runningSubtask.taskName;
  }
  return '';
}

function getFailedReason(details: FragmentType<typeof TaskDescriptionFragmentDoc>[]) {
  const runningSubtask = details
    .map((subtask) => useFragment(TaskDescriptionFragmentDoc, subtask))
    .find((subtask) => subtask.__typename === 'JobSubTask' && subtask.state === QueueTaskState.Failed);

  if (runningSubtask?.__typename === 'JobSubTask') {
    return runningSubtask.taskName;
  }
  return '';
}

function mapProgression({
  compressedFileSize,
  newCompressedFileSize,
  fileSize,
  newFileSize,
  fileCount,
  newFileCount,
  errorCount,
  progressCurrent,
  progressMax,
  speed,
  percent,
}: Partial<JobProgression> | undefined = {}) {
  return {
    compressedFileSize,
    newCompressedFileSize,

    fileSize,
    newFileSize,

    fileCount,
    newFileCount,

    errorCount,

    progressCurrent,
    progressMax,

    speed,
    percent,
  };
}

function toDetailTask(subtask: FragmentType<typeof BackupTaskFragmentDoc>) {
  const subtaskFragment = useFragment(BackupTaskFragmentDoc, subtask);
  if (subtaskFragment.__typename === 'JobGroupTasks') {
    const runningSubtask = getRunningSubtask(subtaskFragment.taskDescription);
    const failedSubtask = getFailedReason(subtaskFragment.taskDescription);

    let description = '';
    if (runningSubtask) {
      description = `${runningSubtask}`;
    } else if (failedSubtask) {
      description = `Failed: ${failedSubtask}`;
    }

    const title = TASKS_GROUP_NAME_TO_DISPLAY[subtaskFragment.groupName ?? ''] ?? (() => subtaskFragment.groupName);

    return {
      title: title(subtaskFragment.description),
      description,
      state: subtaskFragment.state,
      progression: mapProgression(useFragment(ProgressTaskFragmentDoc, subtaskFragment.progression) ?? undefined),
    };
  } else if (subtaskFragment.__typename === 'JobSubTask') {
    const title = TASKS_GROUP_NAME_TO_DISPLAY[subtaskFragment.taskName ?? ''] ?? (() => subtaskFragment.taskName);

    return {
      title: title(subtaskFragment.description),
      description: '',
      state: subtaskFragment.state,
      progression: mapProgression(useFragment(ProgressTaskFragmentDoc, subtaskFragment.progression) ?? undefined),
    };
  } else {
    return {};
  }
}

function toBackupTask(job: FragmentType<typeof JobFragmentDoc>): JobTaskGui {
  const jobFragment = useFragment(JobFragmentDoc, job);
  return {
    jobType: BACKUP_TYPE[jobFragment.name],
    jobId: jobFragment.id ?? 'unknownId',
    hostname: jobFragment.data.host ?? undefined,
    number: jobFragment.data.number ?? undefined,
    ip: jobFragment.data.ip ?? undefined,
    startDate: jobFragment.data.startDate ?? undefined,
    state: jobFragment.data.state,
    failedReason: jobFragment.failedReason ?? undefined,
    progression: mapProgression(useFragment(JobProgressionFragmentDoc, jobFragment.data.progression) ?? undefined),
    details: jobFragment.data.subtasks.map((subtask) => toDetailTask(subtask)),
  };
}

export function useTasks(
  taskFilter: Ref<string[]>,
  queueName: Ref<string | undefined>,
  refetch?: UseQueryReturn<unknown, never>['refetch'],
) {
  const variables = computed(() => ({
    input: {
      states: taskFilter.value,
      queueName: queueName.value,
    },
  }));
  const { result: data, loading: isFetching, subscribeToMore } = useQuery(TasksDocument, variables);

  const tasks = computed(() => data.value?.queue.map((job) => toBackupTask(job)) ?? []);

  subscribeToMore(() => ({
    document: QueueTasksJobUpdatedDocument,
    updateQuery: (previousResult, { subscriptionData }) => {
      if (!subscriptionData.data.jobUpdated) return previousResult;
      const jobFragment = useFragment(JobFragmentDoc, subscriptionData.data.jobUpdated);

      const queue = [...(previousResult?.queue || [])];
      const index = queue.findIndex((task) => {
        const taskFragment = useFragment(JobFragmentDoc, task);
        return taskFragment.id === jobFragment.id;
      });

      if (taskFilter.value.includes(jobFragment.state || '') || taskFilter.value.length === 0) {
        if (index < 0) {
          queue.push(subscriptionData.data.jobUpdated);
          refetch?.();
        }
      } else if (index >= 0) {
        queue.splice(index, 1);
        refetch?.();
      }

      return {
        ...previousResult,
        queue,
      };
    },
  }));

  return {
    tasks,
    isFetching,
  };
}
