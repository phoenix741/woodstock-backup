import type { HostCountByState } from '@/components/hosts/hosts.interface';
import { getState, type DeviceBackupStatus } from '@/components/hosts/hosts.utils';
import { graphql } from '@/generated';
import { HostDocument, HostsDocument } from '@/generated/graphql';
import { useMutation, useQuery } from '@vue/apollo-composable';
import { computed } from 'vue';

export function useDevices() {
  const { result: devices, loading: isDeviceFetching } = useQuery(HostsDocument);

  const devicesByState = computed<HostCountByState[]>(() => {
    const stateCount: Record<DeviceBackupStatus, number> =
      devices.value?.hosts.reduce(
        (acc, device) => {
          const state = getState(device);
          acc[state] = (acc[state] || 0) + 1;
          return acc;
        },
        {} as Record<DeviceBackupStatus, number>,
      ) || ({} as Record<DeviceBackupStatus, number>);

    return Object.entries(stateCount).map(([name, value]) => ({
      name: name as DeviceBackupStatus,
      value,
    }));
  });

  const { mutate } = useMutation(
    graphql(/* GraphQL */ `
      mutation clearCache {
        clearCache {
          void
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

export function useDevice(hostname: string) {
  const { result, loading: isDeviceFetching } = useQuery(HostDocument, {
    hostname,
  });

  const device = computed(() => result.value?.host);

  return {
    device,
    isDeviceFetching,
  };
}
