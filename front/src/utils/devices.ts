import type { HostCountByState } from '@/components/hosts/hosts.interface';
import { getState, getStateKey, type DeviceBackupStatus } from '@/components/hosts/hosts.utils';
import { graphql } from '@/generated';
import { HostDocument, HostsDocument } from '@/generated/graphql';
import { useMutation, useQuery } from '@vue/apollo-composable';
import { computed } from 'vue';

/**
 * Composable for managing devices/hosts data
 */
export function useDevices() {
  const { result: devices, loading: isDeviceFetching } = useQuery(HostsDocument, null, {
    // Lightweight refresh to detect host additions/removals
    pollInterval: 30000,
    fetchPolicy: 'cache-and-network',
  });

  /**
   * Count devices by their backup status state
   * Groups devices with the same state together
   */
  const devicesByState = computed<HostCountByState[]>(() => {
    const stateCount: Map<string, { state: DeviceBackupStatus; count: number }> = new Map();

    devices.value?.hosts.forEach((device) => {
      const state = getState(device);
      const key = getStateKey(state);

      const existing = stateCount.get(key);
      if (existing) {
        existing.count++;
      } else {
        stateCount.set(key, { state, count: 1 });
      }
    });

    return Array.from(stateCount.values()).map(({ state, count }) => ({
      name: state,
      value: count,
    }));
  });

  const { mutate } = useMutation(
    graphql(/* GraphQL */ `
      mutation clearCache {
        clearCache {
          id
        }
      }
    `),
  );

  const clearCache = () => mutate({});

  return {
    devices,
    isDeviceFetching,
    devicesByState,
    clearCache,
  };
}

/**
 * Composable for fetching a single device by hostname
 */
export function useDevice(hostname: string) {
  const { result, loading: isDeviceFetching } = useQuery(
    HostDocument,
    { hostname },
    {
      // 15s polling: availabilityState and timeSinceLastBackup are computed server-side,
      // not event-driven — polling is the only reliable option
      pollInterval: 15000,
      fetchPolicy: 'cache-and-network',
    },
  );

  const device = computed(() => result.value?.host);

  return {
    device,
    isDeviceFetching,
  };
}
