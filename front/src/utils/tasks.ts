import { graphql, useFragment } from '@/generated';
import type { UseQueryReturn } from '@vue/apollo-composable';
import { useQuery } from '@vue/apollo-composable';
import type { Ref } from 'vue';
import { computed } from 'vue';
import { JobFragmentDoc } from './tasks.fragment';

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

      // Vérifier que previousResult existe et a une propriété queue
      if (!previousResult) return previousResult;

      // Créer une copie du résultat précédent
      const updatedQueue = previousResult.queue ? [...previousResult.queue] : [];

      // Trouver l'index du job dans la queue si présent
      const index = updatedQueue.findIndex((task) => {
        if (!task) return false;

        return task.id === subscriptionData.data.jobUpdated.id;
      });

      // Gérer la mise à jour selon l'état du job et le filtre
      const jobState = subscriptionData.data.jobUpdated.state;
      if (taskFilter.value.includes(jobState || '') || taskFilter.value.length === 0) {
        if (index < 0) {
          // Ajouter le job à la queue s'il n'existe pas déjà
          updatedQueue.push(subscriptionData.data.jobUpdated);
          refetch?.();
        } else {
          // Mettre à jour le job existant
          updatedQueue[index] = subscriptionData.data.jobUpdated;
        }
      } else if (index >= 0) {
        // Supprimer le job s'il ne correspond plus aux filtres
        updatedQueue.splice(index, 1);
        refetch?.();
      }

      // Retourner un nouvel objet avec la queue mise à jour
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
