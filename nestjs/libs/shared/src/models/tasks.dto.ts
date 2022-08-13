import { Field, Int, ObjectType, registerEnumType } from '@nestjs/graphql';
import { ApiProperty } from '@nestjs/swagger';
import { Transform } from 'class-transformer';
import { HostConfiguration } from './host-configuration.dto.js';

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
    this.progressCurrent = 0n;
    this.progressMax = 0n;

    Object.assign(this, s);
  }

  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  compressedFileSize: bigint;
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  newCompressedFileSize: bigint;

  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  fileSize: bigint;
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  newFileSize: bigint;

  newFileCount: number;
  fileCount: number;

  speed: number;

  @ApiProperty({ type: 'integer' })
  @Field(() => Int)
  get percent(): number {
    if (this.progressMax) {
      return Number((this.progressCurrent * 100n) / this.progressMax);
    }
    return 0;
  }

  set percent(v: number) {
    this.progressMax = this.progressMax || (v > 0 ? 100n : 0n);
    this.progressCurrent = (BigInt(v) * this.progressMax) / 100n;
  }

  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  progressCurrent: bigint;
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  progressMax: bigint;

  toJSON() {
    return {
      ...this,
      percent: this.percent,
    };
  }
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

  force?: boolean;

  subtasks?: BackupSubTask[];

  state?: BackupState;
  progression?: TaskProgression;

  @ApiProperty({ type: Boolean })
  @Field(() => Boolean)
  complete?: boolean;
}

@ObjectType()
export class Job {
  id!: string;

  name!: string;
  data!: BackupTask;

  delay!: number;

  timestamp!: number;

  @Field(() => Int)
  attemptsMade!: number;

  failedReason?: string;
  stacktrace?: string[];

  finishedOn?: number;
  processedOn?: number;
}
