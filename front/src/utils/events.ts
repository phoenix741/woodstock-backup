import { EventsDocument } from '@/generated/graphql';
import { useQuery } from '@vue/apollo-composable';
import { computed, Ref } from 'vue';

export function useEvents(startDate: Ref<Date>, endDate: Ref<Date>) {
  const startDateString = computed(() => startDate.value.toISOString().split('T')[0]);
  const endDateString = computed(() => endDate.value.toISOString().split('T')[0]);

  const variables = computed(() => ({
    firstEvent: startDateString.value,
    lastEvent: endDateString.value,
  }));

  const { result: data, loading: isFetching } = useQuery(EventsDocument, variables);

  const events = computed(() => data.value?.events);

  return {
    events,
    isFetching,
  };
}
