import { graphql } from '@/generated/gql';
import { useQuery } from '@vue/apollo-composable';
import { computed } from 'vue';
import type { Ref } from 'vue';

const EVENTS_PAGE_SIZE = 50;

const allEventsDocument = graphql(/* GraphQL */ `
  query Events($firstEvent: DateTime!, $lastEvent: DateTime!, $limit: Int, $offset: Int) {
    events(firstEvent: $firstEvent, lastEvent: $lastEvent, limit: $limit, offset: $offset) {
      ...ApplicationEvent
    }
  }
`);

export function useEvents(startDate: Ref<Date>, endDate: Ref<Date>, page?: Ref<number>) {
  const variables = computed(() => ({
    firstEvent: startDate.value,
    lastEvent: endDate.value,
    limit: EVENTS_PAGE_SIZE,
    offset: ((page?.value ?? 1) - 1) * EVENTS_PAGE_SIZE,
  }));

  const { result: data, loading: isFetching } = useQuery(allEventsDocument, variables);

  const events = computed(() => data.value?.events);

  return {
    events,
    isFetching,
    pageSize: EVENTS_PAGE_SIZE,
  };
}
