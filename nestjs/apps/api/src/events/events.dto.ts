import { createUnionType, Field, ObjectType, registerEnumType } from '@nestjs/graphql';

export enum EventType {
  Backup,
  Restore,
  Delete,
  RefcntChecked,
  PoolChecked,
  ChecksumChecked,
  PoolCleaned,
}

export enum EventStep {
  Start,
  End,
}

export enum EventSource {
  User,
  Woodstock,
  Import,
  Cli,
}

export enum EventStatus {
  None,
  Success,
  ClientDisconnected,
  ServerCrashed,
  GenericError,
}

@ObjectType()
export class EventBackupInformation {
  hostname: string;
  number: number;
  sharePath: string[];

  constructor(e: EventBackupInformation) {
    Object.assign(this, e);
  }
}

@ObjectType()
export class EventRefCountInformation {
  fix: boolean;
  count: number;
  error: number;

  constructor(e: EventRefCountInformation) {
    Object.assign(this, e);
  }
}

@ObjectType()
export class EventPoolInformation {
  fix: boolean;
  inUnused: number;
  inRefcnt: number;
  inNothing: number;
  missing: number;

  constructor(e: EventPoolInformation) {
    Object.assign(this, e);
  }
}

@ObjectType()
export class EventPoolCleanedInformation {
  size: number;
  count: number;

  constructor(e: EventPoolCleanedInformation) {
    Object.assign(this, e);
  }
}

export const EventInformation = createUnionType({
  name: 'EventInformation',
  types: () =>
    [EventBackupInformation, EventRefCountInformation, EventPoolInformation, EventPoolCleanedInformation] as const,
});

@ObjectType()
export class ApplicationEvent {
  uuid: string;
  type: EventType;
  step: EventStep;
  source: EventSource;
  timestamp: Date;
  errorMessages: string[];
  status: EventStatus;

  @Field(() => EventInformation)
  information?: typeof EventInformation;

  constructor(e: ApplicationEvent) {
    Object.assign(this, e);
  }
}

registerEnumType(EventType, { name: 'EventType' });
registerEnumType(EventStep, { name: 'EventStep' });
registerEnumType(EventSource, { name: 'EventSource' });
registerEnumType(EventStatus, { name: 'EventStatus' });
