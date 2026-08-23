import { graphql } from '@/generated/gql';
import type { EventsFilterInput } from '@/generated/graphql';
import { useQuery } from '@vue/apollo-composable';
import { computed } from 'vue';
import type { Ref } from 'vue';

const EVENTS_PAGE_SIZE = 50;

const allEventsDocument = graphql(/* GraphQL */ `
  query Events($firstEvent: DateTime!, $lastEvent: DateTime!, $filter: EventsFilterInput, $limit: Int, $offset: Int) {
    events(firstEvent: $firstEvent, lastEvent: $lastEvent, filter: $filter, limit: $limit, offset: $offset) {
      items {
        ...MergedApplicationEvent
      }
      totalCount
    }
  }
`);

export function useEvents(
  startDate: Ref<Date>,
  endDate: Ref<Date>,
  page?: Ref<number>,
  filter?: Ref<EventsFilterInput | undefined>,
) {
  const variables = computed(() => ({
    firstEvent: startDate.value,
    lastEvent: endDate.value,
    filter: filter?.value,
    limit: EVENTS_PAGE_SIZE,
    offset: ((page?.value ?? 1) - 1) * EVENTS_PAGE_SIZE,
  }));

  const { result: data, loading: isFetching } = useQuery(allEventsDocument, variables);

  const events = computed(() => data.value?.events.items);
  const totalCount = computed(() => data.value?.events.totalCount ?? 0);

  return {
    events,
    totalCount,
    isFetching,
    pageSize: EVENTS_PAGE_SIZE,
  };
}
