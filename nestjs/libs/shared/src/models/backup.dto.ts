import { Field, ObjectType, registerEnumType } from '@nestjs/graphql';
import { JsBackup, JsBackupStatus } from '@woodstock/shared-rs';
import { Transform } from 'class-transformer';

registerEnumType(JsBackupStatus, { name: 'BackupStatus' });

@ObjectType()
export class Backup implements JsBackup {
  number!: number;
  status!: JsBackupStatus;

  errorCount!: number;

  startDate!: number;
  endDate?: number;

  fileCount!: number;
  newFileCount!: number;
  existingFileCount!: number;
  removedFileCount!: number;
  modifiedFileCount!: number;

  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  fileSize!: bigint;
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  existingFileSize!: bigint;
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  newFileSize!: bigint;
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  modifiedFileSize!: bigint;

  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  compressedFileSize!: bigint;
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  existingCompressedFileSize!: bigint;
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  newCompressedFileSize!: bigint;
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  modifiedCompressedFileSize!: bigint;

  speed!: number;

  agentVersion?: string;
}
