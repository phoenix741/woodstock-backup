import { graphql } from '@/generated';
import { useQuery } from '@vue/apollo-composable';
import { computed } from 'vue';

export function useServerInformation() {
  const { result: data, loading: isFetching } = useQuery(
    graphql(/* GraphQL */ `
      query ServerInformations {
        informations {
          uptime
          hostname
          woodstockVersion
        }
      }
    `),
    {},
  );

  const informations = computed(() => data.value?.informations);

  return {
    informations,
    isFetching,
  };
}
