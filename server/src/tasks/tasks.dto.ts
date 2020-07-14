import { HostConfiguration } from '../hosts/host-configuration.dto';
import { ApiProperty } from '@nestjs/swagger';
import { ObjectType, registerEnumType, Field, Int } from '@nestjs/graphql';

export enum BackupState {
  WAITING = 'WAITING',
  RUNNING = 'RUNNING',
  SUCCESS = 'SUCCESS',
  ABORTED = 'ABORTED',
  FAILED = 'FAILED',
}

registerEnumType(BackupState, {
  name: 'BackupState',
});

@ObjectType()
export class TaskProgression {
  fileSize = 0;
  newFileSize = 0;

  newFileCount = 0;
  fileCount = 0;

  speed = 0;
  percent = 0;
}

@ObjectType()
export class BackupSubTask {
  context!: string;
  description!: string;
  state!: BackupState;
  progression?: TaskProgression;
}

@ObjectType()
export class BackupTask {
  host!: string;
  config?: HostConfiguration;
  previousNumber?: number;
  number?: number;
  ip?: string;
  startDate?: number;

  subtasks?: BackupSubTask[];
  state?: BackupState;
  progression?: TaskProgression;

  @ApiProperty({ type: Boolean })
  @Field(() => Boolean)
  complete?: boolean;
}

@ObjectType()
export class Job {
  @Field(() => Int)
  id!: number;

  name!: string;
  data!: BackupTask;

  @Field(() => Int)
  delay!: number;

  @Field(() => Int)
  timestamp!: number;

  @Field(() => Int)
  attemptsMade!: number;

  failedReason?: string;
  stacktrace?: string[];

  @Field(() => Int)
  finishedOn?: number;

  @Field(() => Int)
  processedOn?: number;
}
