import { HostCountByState } from '@/components/hosts/hosts.interface';
import { getState } from '@/components/hosts/hosts.utils';
import { HostsDocument } from '@/generated/graphql';
import { useQuery } from '@vue/apollo-composable';
import { computed } from 'vue';

export function useDevices() {
  const { result: devices, loading: isDeviceFetching } = useQuery(HostsDocument);

  const devicesByState = computed<HostCountByState[]>(() => {
    const devicesByState =
      devices.value?.hosts.reduce((acc, device) => {
        const state = getState(device);
        acc[state] = (acc[state] || 0) + 1;
        return acc;
      }, {} as Record<string, number>) || {};

    return Object.entries(devicesByState).map(([name, value]) => ({
      name,
      value,
    }));
  });

  return {
    devices,
    isDeviceFetching,
    devicesByState,
  };
}
