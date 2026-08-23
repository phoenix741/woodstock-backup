import { EventSource, EventStatus, EventType } from '@/generated/graphql';

const eventTypeLabels: Record<EventType, string> = {
  [EventType.Backup]: 'Backup',
  [EventType.Restore]: 'Restore',
  [EventType.Delete]: 'Delete',
  [EventType.PoolChecked]: 'Pool Checked',
  [EventType.PoolCleaned]: 'Pool Cleaned',
  [EventType.HashConversion]: 'Hash Conversion',
};

const eventStatusLabels: Record<EventStatus, string> = {
  [EventStatus.None]: 'None',
  [EventStatus.Success]: 'Success',
  [EventStatus.ClientDisconnected]: 'Client Disconnected',
  [EventStatus.ServerCrashed]: 'Server Crashed',
  [EventStatus.GenericError]: 'Generic Error',
  [EventStatus.Cancelled]: 'Cancelled',
  [EventStatus.Aborted]: 'Aborted',
};

const eventSourceLabels: Record<EventSource, string> = {
  [EventSource.User]: 'User',
  [EventSource.Woodstock]: 'Woodstock',
  [EventSource.Import]: 'Import',
  [EventSource.Cli]: 'CLI',
};

export function eventTypeLabel(type: EventType): string {
  return eventTypeLabels[type] ?? type;
}

export function eventStatusLabel(status: EventStatus): string {
  return eventStatusLabels[status] ?? status;
}

export function eventSourceLabel(source: EventSource): string {
  return eventSourceLabels[source] ?? source;
}

export const eventTypeOptions = Object.values(EventType).map((value) => ({
  title: eventTypeLabel(value),
  value,
}));

export const eventStatusOptions = Object.values(EventStatus).map((value) => ({
  title: eventStatusLabel(value),
  value,
}));

export const eventSourceOptions = Object.values(EventSource).map((value) => ({
  title: eventSourceLabel(value),
  value,
}));
