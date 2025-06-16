import type { DeviceBackupStatus } from './hosts.utils';

export interface HostCountByState {
  name: DeviceBackupStatus;
  value: number;
}

export interface HostBySize {
  name: string;
  value: bigint;
}
