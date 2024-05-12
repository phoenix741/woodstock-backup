import { Field, Int, ObjectType } from '@nestjs/graphql';
import { Transform } from 'class-transformer';

@ObjectType()
export class BigIntTimeSerie {
  time: number;

  @Transform((v) => v.value && BigInt(v.value))
  @Field(() => BigInt)
  value: bigint;

  constructor(o: Partial<BigIntTimeSerie>) {
    Object.assign(this, o);
  }
}

@ObjectType()
export class NumberTimeSerie {
  time: number;

  @Field(() => Int)
  value: number;

  constructor(o: NumberTimeSerie) {
    Object.assign(this, o);
  }
}

@ObjectType()
export class DiskUsage {
  @Transform((v) => v.value && BigInt(v.value))
  @Field(() => BigInt)
  used?: bigint;
  usedRange?: BigIntTimeSerie[];
  @Transform((v) => v.value && BigInt(v.value))
  @Field(() => BigInt)
  usedLastMonth?: bigint;

  @Transform((v) => v.value && BigInt(v.value))
  @Field(() => BigInt)
  free?: bigint;
  freeRange?: BigIntTimeSerie[];
  @Transform((v) => v.value && BigInt(v.value))
  @Field(() => BigInt)
  freeLastMonth?: bigint;

  @Transform((v) => v.value && BigInt(v.value))
  @Field(() => BigInt)
  total?: bigint;
  totalRange?: BigIntTimeSerie[];
  @Transform((v) => v.value && BigInt(v.value))
  @Field(() => BigInt)
  totalLastMonth?: bigint;
}

@ObjectType()
export class PoolUsage {
  @Field(() => Int)
  longestChain?: number;
  longestChainRange?: NumberTimeSerie[];
  @Field(() => Int)
  longestChainLastMonth?: number;

  @Field(() => Int)
  nbChunk?: number;
  nbChunkRange?: NumberTimeSerie[];
  @Field(() => Int)
  nbChunkLastMonth?: number;

  @Field(() => Int)
  nbRef?: number;
  nbRefRange?: NumberTimeSerie[];
  @Field(() => Int)
  nbRefLastMonth?: number;

  @Transform((v) => v.value && BigInt(v.value))
  @Field(() => BigInt)
  size?: bigint;
  sizeRange?: BigIntTimeSerie[];
  @Transform((v) => v.value && BigInt(v.value))
  @Field(() => BigInt)
  sizeLastMonth?: bigint;

  @Transform((v) => v.value && BigInt(v.value))
  @Field(() => BigInt)
  compressedSize?: bigint;
  compressedSizeRange?: BigIntTimeSerie[];
  @Transform((v) => v.value && BigInt(v.value))
  @Field(() => BigInt)
  compressedSizeLastMonth?: bigint;

  @Transform((v) => v.value && BigInt(v.value))
  @Field(() => BigInt)
  unusedSize?: bigint;
  unusedSizeRange?: BigIntTimeSerie[];
  @Transform((v) => v.value && BigInt(v.value))
  @Field(() => BigInt)
  unusedSizeLastMonth?: bigint;
}

@ObjectType()
export class HostStatistics {
  host?: string;

  @Field(() => Int)
  longestChain?: number;
  longestChainRange?: NumberTimeSerie[];
  @Field(() => Int)
  longestChainLastMonth?: number;

  @Field(() => Int)
  nbChunk?: number;
  nbChunkRange?: NumberTimeSerie[];
  @Field(() => Int)
  nbChunkLastMonth?: number;

  @Field(() => Int)
  nbRef?: number;
  nbRefRange?: NumberTimeSerie[];
  @Field(() => Int)
  nbRefLastMonth?: number;

  @Transform((v) => v.value && BigInt(v.value))
  @Field(() => BigInt)
  size?: bigint;
  sizeRange?: BigIntTimeSerie[];
  @Transform((v) => v.value && BigInt(v.value))
  @Field(() => BigInt)
  sizeLastMonth?: bigint;

  @Transform((v) => v.value && BigInt(v.value))
  @Field(() => BigInt)
  compressedSize?: bigint;
  compressedSizeRange?: BigIntTimeSerie[];
  @Transform((v) => v.value && BigInt(v.value))
  @Field(() => BigInt)
  compressedSizeLastMonth?: bigint;
}

@ObjectType()
export class Statistics {
  diskUsage?: DiskUsage;
  poolUsage?: PoolUsage;
  hosts?: HostStatistics[];
}
