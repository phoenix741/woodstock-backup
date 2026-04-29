import { graphql, useFragment } from '@/generated';
import type { UseQueryReturn } from '@vue/apollo-composable';
import { useQuery } from '@vue/apollo-composable';
import type { Ref } from 'vue';
import { computed } from 'vue';
import { JobFragmentDoc } from './tasks.fragment';
import type { JobStatus } from '@/generated/graphql';

/** Simple debounce without an external dependency */
function createDebounce<Args extends unknown[]>(fn: (...args: Args) => void, delay: number): (...args: Args) => void {
  let timer: ReturnType<typeof setTimeout> | null = null;
  return (...args: Args) => {
    if (timer !== null) clearTimeout(timer);
    timer = setTimeout(() => fn(...args), delay);
  };
}

export function useTasks(
  taskFilter: Ref<JobStatus | undefined>,
  queueName: Ref<string | undefined>,
  refetch?: UseQueryReturn<unknown, never>['refetch'],
) {
  const variables = computed(() => ({
    input: {
      state: taskFilter.value,
      queueName: queueName.value,
    },
  }));
  const {
    result: data,
    loading: isFetching,
    subscribeToMore,
  } = useQuery(
    graphql(/* GraphQL */ `
      query Tasks($input: QueueListInput!) {
        queue(input: $input) {
          ...Job
        }
      }
    `),
    variables,
  );

  const tasks = computed(() => data.value?.queue.map((job) => useFragment(JobFragmentDoc, job)) ?? []);

  // Debounce: avoids one HTTP request per WebSocket event (burst of jobs)
  const debouncedRefetch = refetch ? createDebounce(refetch, 500) : undefined;

  // Note: subscribeToMore from @vue/apollo-composable returns void (no unsubscribe handle exposed)
  subscribeToMore(() => ({
    document: graphql(/* GraphQL */ `
      subscription QueueTasksJobUpdated {
        jobUpdated {
          ...Job
        }
      }
    `),
    updateQuery: (previousResult, { subscriptionData }) => {
      if (!subscriptionData.data?.jobUpdated) return previousResult;
      if (!previousResult) return previousResult;

      const updatedQueue = previousResult.queue ? [...previousResult.queue] : [];

      const index = updatedQueue.findIndex((task) => {
        if (!task) return false;
        return task.jobId === subscriptionData.data.jobUpdated.jobId;
      });

      // Capture the filter at execution time (not at creation time)
      const currentFilter = taskFilter.value;
      const jobState = subscriptionData.data.jobUpdated.status;

      if (!currentFilter || currentFilter.includes(jobState || '') || currentFilter.length === 0) {
        if (index < 0) {
          updatedQueue.push(subscriptionData.data.jobUpdated);
          // Debounced: single HTTP request even if 100 jobs arrive at once
          debouncedRefetch?.();
        } else {
          updatedQueue[index] = subscriptionData.data.jobUpdated;
        }
      } else if (index >= 0) {
        updatedQueue.splice(index, 1);
        debouncedRefetch?.();
      }

      return {
        ...previousResult,
        queue: updatedQueue,
      };
    },
  }));

  return {
    tasks,
    isFetching,
  };
}
