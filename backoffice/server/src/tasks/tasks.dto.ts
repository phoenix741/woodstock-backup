import { HostConfiguration } from '../hosts/host-configuration.dto';
import { ApiProperty } from '@nestjs/swagger';
import { ObjectType, registerEnumType, Field, Int } from '@nestjs/graphql';
import { Transform } from 'class-transformer';
import { BigIntScalar } from 'src/utils/bigint.scalar';

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
  constructor(s: Partial<TaskProgression> = {}) {
    this.compressedFileSize = 0n;
    this.newCompressedFileSize = 0n;

    this.fileSize = 0n;
    this.newFileSize = 0n;

    this.newFileCount = 0;
    this.fileCount = 0;

    this.speed = 0;
    this.percent = 0;

    Object.assign(this, s);
  }

  @ApiProperty({ type: 'integer' })
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  compressedFileSize: bigint;
  @ApiProperty({ type: 'integer' })
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  newCompressedFileSize: bigint;

  @ApiProperty({ type: 'integer' })
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  fileSize: bigint;
  @ApiProperty({ type: 'integer' })
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  newFileSize: bigint;

  newFileCount: number;
  fileCount: number;

  speed: number;
  percent: number;
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
  originalStartDate?: number;

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
