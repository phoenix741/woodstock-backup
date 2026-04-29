import { graphql } from '@/generated';
import { useQuery } from '@vue/apollo-composable';
import { computed } from 'vue';

export function useDiskUsageStats() {
  const { result: data, loading: isStatsFetching } = useQuery(
    graphql(/* GraphQL */ `
      query DiskUsageStatistics {
        statistics {
          hosts {
            host

            size
            compressedSize
          }
        }
      }
    `),
  );

  const devicesBySize = computed(() => {
    const devicesBySize =
      data.value?.statistics?.hosts?.reduce(
        (acc, device) => {
          const host = device.host ?? 'unknown';
          acc[host] = (acc[host] ?? 0n) + (device.compressedSize ?? 0n);
          return acc;
        },
        {} as Record<string, bigint>,
      ) || {};

    return Object.entries(devicesBySize).map(([name, value]) => ({
      name,
      value,
    }));
  });

  return {
    devicesBySize,
    isStatsFetching,
  };
}

export function useQueueRealtimeStats() {
  const {
    result: data,
    loading: isFetching,
    refetch,
  } = useQuery(
    graphql(/* GraphQL */ `
      query QueueStatistics {
        queueStats {
          pending
          running
          success
          failed
          dead
        }
      }
    `),
    // Fallback polling: stats remain up-to-date even without an active subscription
    null,
    { pollInterval: 10000, fetchPolicy: 'cache-and-network' },
  );

  const queueStats = computed(() => {
    return (
      data.value?.queueStats || {
        running: 0,
        pending: 0,
        failed: 0,
        dead: 0,
        success: 0,
      }
    );
  });

  return {
    queueStats,
    isFetching,
    refetch,
  };
}

export function usePoolStats() {
  const { result, loading: isFetching } = useQuery(
    graphql(/* GraphQL */ `
      query PoolStatistics {
        statistics {
          diskUsage {
            used
            usedLastMonth
            free
            total
          }
          poolUsage {
            nbChunk
            nbChunkLastMonth
            nbChunkRange {
              time
              value
            }

            nbRef
            nbRefLastMonth

            size
            compressedSize
            compressedSizeLastMonth
            compressedSizeRange {
              time
              value
            }

            unusedSize
          }
        }
      }
    `),
    null,
    // Automatic refresh every 30s
    // + immediate update via cache if data is available
    { pollInterval: 30000, fetchPolicy: 'cache-and-network' },
  );

  return {
    result,
    isFetching,
  };
}
