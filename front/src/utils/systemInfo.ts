import { graphql } from '@/generated';
import { useQuery } from '@vue/apollo-composable';
import { computed } from 'vue';

export function useSystemInfo() {
  const { result: data, loading: isFetching } = useQuery(
    graphql(/* GraphQL */ `
      query SystemInfo {
        informations {
          hostname
          woodstockVersion
          services {
            serviceType
            instanceId
            version
            hostname
            startedAt
          }
        }
      }
    `),
    {},
  );

  const informations = computed(() => data.value?.informations);
  const services = computed(() => informations.value?.services ?? []);

  return {
    informations,
    services,
    isFetching,
  };
}
