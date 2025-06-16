import { Field, Int, ObjectType, registerEnumType } from '@nestjs/graphql';
import {
  JsCleanerErrorState,
  JsCleanerExecutionState,
  JsCleanerProgression,
  JsCleanerStatusUpdate,
  JsEventPoolCleanedInformation,
} from '@woodstock/shared-rs';
import { Transform } from 'class-transformer';

// --- ENUMS ---

registerEnumType(JsCleanerExecutionState, { name: 'CleanerExecutionState' });
registerEnumType(JsCleanerErrorState, { name: 'CleanerErrorState' });

// --- OBJECTS ---

@ObjectType()
export class CleanerProgression implements JsCleanerProgression {
  @Field(() => Int)
  progressMax: number;

  @Field(() => Int)
  progressCurrent: number;

  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  fileSize: bigint;

  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  compressedFileSize: bigint;
}

@ObjectType()
export class CleanerTaskState implements JsCleanerStatusUpdate {
  @Field(() => JsCleanerExecutionState)
  executionState: JsCleanerExecutionState;

  @Field(() => JsCleanerErrorState, { nullable: true })
  errorState?: JsCleanerErrorState;

  @Field({ nullable: true })
  errorMessage?: string;

  @Field(() => CleanerProgression)
  progression: CleanerProgression;
}

@ObjectType()
export class PoolCleanedInformation implements JsEventPoolCleanedInformation {
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  size: bigint;

  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  count: number;
}
