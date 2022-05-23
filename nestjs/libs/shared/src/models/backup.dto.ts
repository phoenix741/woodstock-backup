import { Field, ObjectType } from '@nestjs/graphql';
import { ApiProperty } from '@nestjs/swagger';
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

  @ApiProperty({ type: 'integer' })
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  fileSize!: bigint;
  @ApiProperty({ type: 'integer' })
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  existingFileSize!: bigint;
  @ApiProperty({ type: 'integer' })
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  newFileSize!: bigint;

  @ApiProperty({ type: 'integer' })
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  compressedFileSize!: bigint;
  @ApiProperty({ type: 'integer' })
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  existingCompressedFileSize!: bigint;
  @ApiProperty({ type: 'integer' })
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  newCompressedFileSize!: bigint;

  speed!: number;
}
