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
  number?: number;
  ip?: string;
  previousDirectory?: string;
  destinationDirectory?: string;
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
  @Field(type => Int)
  id!: number;

  name!: string;
  data!: BackupTask;

  progress!: number;

  @Field(type => Int)
  delay!: number;

  @Field(type => Int)
  timestamp!: number;

  @Field(type => Int)
  attemptsMade!: number;

  failedReason?: string;
  stacktrace?: string[];

  @Field(type => Int)
  finishedOn?: number;

  @Field(type => Int)
  processedOn?: number;
}

@ObjectType()
export class BackupQueue {}
