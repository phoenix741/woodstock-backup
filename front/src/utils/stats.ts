import { DiskUsageStatisticsDocument, PoolStatisticsDocument, QueueStatisticsDocument } from '@/generated/graphql';
import { useQuery } from '@vue/apollo-composable';
import { computed } from 'vue';

export function useDiskUsageStats() {
  const { result: data, loading: isStatsFetching } = useQuery(DiskUsageStatisticsDocument);

  const devicesBySize = computed(() => {
    const devicesBySize =
      data.value?.statistics?.hosts?.reduce((acc, device) => {
        const host = device.host || 'unknown';
        acc[host] = (acc[host] ?? 0n) + (device.compressedSize || 0n);
        return acc;
      }, {} as Record<string, bigint>) || {};

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
  const { result: data, loading: isFetching, refetch } = useQuery(QueueStatisticsDocument);

  const queueStats = computed(() => {
    return (
      data.value?.queueStats || {
        active: 0,
        waiting: 0,
        failed: 0,
        delayed: 0,
        completed: 0,
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
  const { result, loading: isFetching } = useQuery(PoolStatisticsDocument);

  return {
    result,
    isFetching,
  };
}
