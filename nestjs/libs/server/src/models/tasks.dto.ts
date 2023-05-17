import { createUnionType, Field, InputType, Int, ObjectType, registerEnumType } from '@nestjs/graphql';
import { ApiProperty } from '@nestjs/swagger';
import { JobState } from 'bullmq';
import { Transform, Type } from 'class-transformer';
import { Allow } from 'class-validator';
import { QueueTaskState } from '../tasks';

registerEnumType(QueueTaskState, {
  name: 'QueueTaskState',
});

@ObjectType()
export class JobProgression {
  @Transform((v) => v.value && BigInt(v.value))
  @Field(() => BigInt)
  compressedFileSize?: bigint;
  @Transform((v) => v.value && BigInt(v.value))
  @Field(() => BigInt)
  newCompressedFileSize?: bigint;

  @Transform((v) => v.value && BigInt(v.value))
  @Field(() => BigInt)
  fileSize?: bigint;
  @Transform((v) => v.value && BigInt(v.value))
  @Field(() => BigInt)
  newFileSize?: bigint;

  @ApiProperty({ type: 'integer' })
  @Field(() => Int)
  newFileCount?: number;

  @ApiProperty({ type: 'integer' })
  @Field(() => Int)
  fileCount?: number;

  @ApiProperty({ type: 'integer' })
  @Field(() => Int)
  errorCount?: number;

  speed?: number;
  percent?: number;

  @Transform((v) => v.value && BigInt(v.value))
  @Field(() => BigInt)
  progressCurrent?: bigint;
  @Transform((v) => v.value && BigInt(v.value))
  @Field(() => BigInt)
  progressMax?: bigint;
}

@ObjectType()
export class JobSubTask {
  taskName: string;
  state?: QueueTaskState;
  @Type(() => JobProgression)
  progression?: JobProgression;
  description?: string;
}

export const SubTaskOrGroupTasks = createUnionType({
  name: 'SubTaskOrGroupTasks',
  types: () => [JobSubTask, JobGroupTasks] as const,
});

@ObjectType()
export class JobGroupTasks {
  groupName?: string;

  @Field(() => [SubTaskOrGroupTasks])
  subtasks: (typeof SubTaskOrGroupTasks)[];
  state?: QueueTaskState;
  @Type(() => JobProgression)
  progression?: JobProgression;
  description?: string;
}

@ObjectType()
export class BackupTask extends JobGroupTasks {
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

  @Type(() => BackupTask)
  data!: BackupTask;

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
