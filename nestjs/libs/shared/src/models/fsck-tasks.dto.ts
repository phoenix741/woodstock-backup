import { Field, Int, ObjectType, registerEnumType } from '@nestjs/graphql';
import {
  JsChunkProgression,
  JsFsckErrorState,
  JsFsckExecutionState,
  JsFsckStatusUpdate,
  JsRefcntProgression,
  JsUnusedProgression,
} from '@woodstock/shared-rs';

// --- ENUMS ---

registerEnumType(JsFsckExecutionState, { name: 'FsckExecutionState' });
registerEnumType(JsFsckErrorState, { name: 'FsckErrorState' });

// --- OBJECTS ---

@ObjectType()
export class RefcntProgression implements JsRefcntProgression {
  @Field(() => Int)
  progressMax: number;

  @Field(() => Int)
  progressCurrent: number;

  @Field(() => Int)
  errorCount: number;

  @Field(() => Int)
  totalCount: number;
}

@ObjectType()
export class UnusedProgression implements JsUnusedProgression {
  @Field(() => Int)
  progressMax: number;

  @Field(() => Int)
  progressCurrent: number;

  @Field(() => Int)
  inNothing: number;

  @Field(() => Int)
  inRefcnt: number;

  @Field(() => Int)
  inUnused: number;

  @Field(() => Int)
  missing: number;
}

@ObjectType()
export class ChunkProgression implements JsChunkProgression {
  @Field(() => Int)
  progressMax: number;

  @Field(() => Int)
  progressCurrent: number;

  @Field(() => Int)
  errorCount: number;

  @Field(() => Int)
  totalCount: number;
}

@ObjectType()
export class FsckTaskState implements JsFsckStatusUpdate {
  @Field(() => JsFsckExecutionState)
  executionState: JsFsckExecutionState;

  @Field(() => JsFsckErrorState, { nullable: true })
  errorState?: JsFsckErrorState;

  @Field({ nullable: true })
  errorMessage?: string;

  @Field(() => RefcntProgression)
  refcntProgression: RefcntProgression;

  @Field(() => UnusedProgression)
  unusedProgression: UnusedProgression;

  @Field(() => ChunkProgression)
  chunkProgression: ChunkProgression;

  @Field(() => Boolean)
  dryRun: boolean;
}
