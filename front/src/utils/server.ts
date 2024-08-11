import { ServerInformationsDocument } from '@/generated/graphql';
import { useQuery } from '@vue/apollo-composable';
import { computed } from 'vue';

export function useServerInformation() {
  const { result: data, loading: isFetching } = useQuery(ServerInformationsDocument, {});

  const informations = computed(() => data.value?.informations);

  return {
    informations,
    isFetching,
  };
}
