import { Field, Float, Int, ObjectType, registerEnumType } from '@nestjs/graphql';
import {
  JsBackupExecutionState,
  JsBackupProgression,
  JsBackupState,
  JsErrorState,
  JsExecuteCommandExecutionState,
  JsExecuteCommandState,
  JsFileListProgression,
  JsShareExecutionState,
  JsShareState,
} from '@woodstock/shared-rs';
import { Transform } from 'class-transformer';
import { ExecuteCommandOperation } from './host-configuration.dto';

// --- ENUMS ---

registerEnumType(JsBackupExecutionState, { name: 'BackupExecutionState' });
registerEnumType(JsErrorState, { name: 'BackupErrorState' });
registerEnumType(JsExecuteCommandExecutionState, { name: 'ExecuteCommandExecutionState' });
registerEnumType(JsShareExecutionState, { name: 'ShareExecutionState' });

// --- OBJECTS ---

@ObjectType()
export class FileListProgression implements JsFileListProgression {
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  fileSize: bigint;
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  newFileSize: bigint;
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  modifiedFileSize: bigint;
  @Field(() => Int)
  newFileCount: number;
  @Field(() => Int)
  modifiedFileCount: number;
  @Field(() => Int)
  removedFileCount: number;
}

@ObjectType()
export class BackupProgression implements JsBackupProgression {
  @Field(() => Int)
  startDate: number;
  @Field(() => Int, { nullable: true })
  startTransferDate?: number;
  @Field(() => Int, { nullable: true })
  endTransferDate?: number;

  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  fileSize: bigint;
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  newFileSize: bigint;
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  modifiedFileSize: bigint;

  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  compressedFileSize: bigint;
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  newCompressedFileSize: bigint;
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  modifiedCompressedFileSize: bigint;

  @Field(() => Int)
  fileCount: number;
  @Field(() => Int)
  newFileCount: number;
  @Field(() => Int)
  modifiedFileCount: number;
  @Field(() => Int)
  removedFileCount: number;
  @Field(() => Int)
  errorCount: number;

  @Field(() => Float)
  speed: number;
  @Field(() => Float)
  percent: number;

  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  progressCurrent: bigint;
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  progressMax: bigint;
}

@ObjectType()
export class ExecuteCommandState implements JsExecuteCommandState {
  @Field()
  command: ExecuteCommandOperation;
  @Field(() => JsExecuteCommandExecutionState)
  executionState: JsExecuteCommandExecutionState;
}

@ObjectType()
export class ShareState implements JsShareState {
  @Field()
  share: string;
  @Field(() => FileListProgression)
  fileListProgression: FileListProgression;
  @Field(() => BackupProgression)
  backupProgression: BackupProgression;
  @Field(() => JsShareExecutionState)
  executionState: JsShareExecutionState;
}

@ObjectType()
export class BackupTaskState implements JsBackupState {
  @Field(() => JsBackupExecutionState)
  executionState: JsBackupExecutionState;
  @Field(() => JsErrorState, { nullable: true })
  errorState?: JsErrorState;
  @Field({ nullable: true })
  errorMessage?: string;
  @Field(() => BackupProgression)
  progression: BackupProgression;
  @Field(() => [ExecuteCommandState])
  preCommandStates: ExecuteCommandState[];
  @Field(() => [ShareState])
  shareStates: ShareState[];
  @Field(() => [ExecuteCommandState])
  postCommandStates: ExecuteCommandState[];
}
