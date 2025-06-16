import { createUnionType, Field, Int, ObjectType, registerEnumType } from '@nestjs/graphql';
import {
  JsChunkAlgorithm,
  JsEventBackupInformation,
  JsEventPoolCleanedInformation,
  JsEventPoolInformation,
  JsEventSource,
  JsEventStatus,
  JsEventStep,
  JsEventType,
  JsHashConversionInformation,
} from '@woodstock/shared-rs';
import { Transform } from 'class-transformer';

@ObjectType()
export class EventBackupInformation implements JsEventBackupInformation {
  hostname: string;
  @Field(() => Int)
  number: number;
  sharePath: string[];

  constructor(e: JsEventBackupInformation) {
    Object.assign(this, e);
  }
}

@ObjectType()
export class EventPoolInformation implements JsEventPoolInformation {
  fix: boolean;
  @Field(() => Int)
  refcount: number;
  @Field(() => Int)
  refcountError: number;
  @Field(() => Int)
  inUnused: number;
  @Field(() => Int)
  inRefcnt: number;
  @Field(() => Int)
  inNothing: number;
  @Field(() => Int)
  missing: number;
  @Field(() => Int)
  chunkCount: number;
  @Field(() => Int)
  chunkError: number;

  constructor(e: JsEventPoolInformation) {
    Object.assign(this, e);
  }
}

@ObjectType()
export class EventPoolCleanedInformation implements JsEventPoolCleanedInformation {
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  size: bigint;
  @Field(() => Int)
  count: number;

  constructor(e: JsEventPoolCleanedInformation) {
    Object.assign(this, e);
  }
}

@ObjectType()
export class EventHashConversionInformation implements JsHashConversionInformation {
  @Field(() => Int)
  count: number;
  algorithm: JsChunkAlgorithm;

  constructor(e: EventHashConversionInformation) {
    Object.assign(this, e);
  }
}

export const EventInformation = createUnionType({
  name: 'EventInformation',
  types: () =>
    [
      EventBackupInformation,
      EventPoolInformation,
      EventPoolCleanedInformation,
      EventHashConversionInformation,
    ] as const,
});

@ObjectType()
export class ApplicationEvent {
  uuid: string;
  type: JsEventType;
  step: JsEventStep;
  source: JsEventSource;
  timestamp: Date;
  errorMessages: string[];
  status: JsEventStatus;

  @Field(() => EventInformation)
  information?: typeof EventInformation;

  constructor(e: ApplicationEvent) {
    Object.assign(this, e);
  }
}

registerEnumType(JsEventType, { name: 'EventType' });
registerEnumType(JsEventStep, { name: 'EventStep' });
registerEnumType(JsEventSource, { name: 'EventSource' });
registerEnumType(JsEventStatus, { name: 'EventStatus' });
registerEnumType(JsChunkAlgorithm, { name: 'ChunkAlgorithm' });
