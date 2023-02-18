import { createUnionType, Field, Int, ObjectType, registerEnumType } from '@nestjs/graphql';
import { ApiProperty } from '@nestjs/swagger';
import { Transform, Type } from 'class-transformer';
import { QueueTaskState } from '../tasks/queue-tasks.model.js';

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
  state: QueueTaskState;
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
  state: QueueTaskState;
  @Type(() => JobProgression)
  progression?: JobProgression;
  description?: string;
}

@ObjectType()
export class BackupTask extends JobGroupTasks {
  host!: string;
  number?: number;
  ip?: string;
  startDate?: number;
}

@ObjectType()
export class Job {
  id?: string;
  name!: string;
  state: string;

  @Type(() => BackupTask)
  data!: BackupTask;

  @Field(() => Int)
  attemptsMade!: number;
}
