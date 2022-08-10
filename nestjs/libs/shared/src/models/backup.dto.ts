import { Field, ObjectType } from '@nestjs/graphql';
import { Transform } from 'class-transformer';

@ObjectType()
export class Backup {
  number!: number;
  complete!: boolean;

  startDate!: number;
  endDate?: number;

  fileCount!: number;
  newFileCount!: number;
  existingFileCount!: number;

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
  compressedFileSize!: bigint;
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  existingCompressedFileSize!: bigint;
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  newCompressedFileSize!: bigint;

  speed!: number;
}
