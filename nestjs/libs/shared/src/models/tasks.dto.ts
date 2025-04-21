import { Field, InputType, Int, ObjectType } from '@nestjs/graphql';
import { JobState } from 'bullmq';
import { Type } from 'class-transformer';
import { Allow } from 'class-validator';
import { BackupQueueDataUnion, TaskStateUnion } from './task-state.union';

@ObjectType()
export class BackupTask {
  host?: string;
  number?: number;
  ip?: string;
  startDate?: number;
}

@ObjectType()
export class Job {
  id?: string;
  queueName!: string;
  name!: string;
  state: string;

  @Field(() => BackupQueueDataUnion, { nullable: true })
  data!: typeof BackupQueueDataUnion;

  @Field(() => TaskStateUnion, { nullable: true })
  progression?: typeof TaskStateUnion;

  @Field(() => Int)
  attemptsMade!: number;
  failedReason?: string;
}

@InputType()
export class QueueListInput {
  @Field(() => [String], { defaultValue: [] })
  @Type(() => String)
  @Allow()
  states: JobState[];

  @Allow()
  queueName?: string;

  @Allow()
  operationName?: string;
}
