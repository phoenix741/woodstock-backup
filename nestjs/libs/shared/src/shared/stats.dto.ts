import { Field, Float, Int, ObjectType } from '@nestjs/graphql';

@ObjectType()
export class TimeSerie {
  @Field(() => BigInt)
  time: number;

  @Field(() => Float)
  value: number;

  constructor(o: Partial<TimeSerie>) {
    Object.assign(this, o);
  }
}

@ObjectType()
export class DiskUsage {
  @Field(() => Int)
  used?: number;
  usedRange?: TimeSerie[];
  usedLastMonth?: number;

  @Field(() => Int)
  free?: number;
  freeRange?: TimeSerie[];
  freeLastMonth?: number;

  @Field(() => Int)
  total?: number;
  totalRange?: TimeSerie[];
  totalLastMonth?: number;
}

@ObjectType()
export class PoolUsage {
  @Field(() => Int)
  longestChain?: number;
  longestChainRange?: TimeSerie[];
  longestChainLastMonth?: number;

  @Field(() => Int)
  nbChunk?: number;
  nbChunkRange?: TimeSerie[];
  nbChunkLastMonth?: number;

  @Field(() => Int)
  nbRef?: number;
  nbRefRange?: TimeSerie[];
  nbRefLastMonth?: number;

  @Field(() => Int)
  size?: number;
  sizeRange?: TimeSerie[];
  sizeLastMonth?: number;

  @Field(() => Int)
  compressedSize?: number;
  compressedSizeRange?: TimeSerie[];
  compressedSizeLastMonth?: number;
}

@ObjectType()
export class HostStatistics {
  host?: string;

  @Field(() => Int)
  longestChain?: number;
  longestChainRange?: TimeSerie[];
  longestChainLastMonth?: number;

  @Field(() => Int)
  nbChunk?: number;
  nbChunkRange?: TimeSerie[];
  nbChunkLastMonth?: number;

  @Field(() => Int)
  nbRef?: number;
  nbRefRange?: TimeSerie[];
  nbRefLastMonth?: number;

  @Field(() => Int)
  size?: number;
  sizeRange?: TimeSerie[];
  sizeLastMonth?: number;

  @Field(() => Int)
  compressedSize?: number;
  compressedSizeRange?: TimeSerie[];
  compressedSizeLastMonth?: number;
}

@ObjectType()
export class Statistics {
  diskUsage?: DiskUsage;
  poolUsage?: PoolUsage;
  hosts?: HostStatistics[];
}
