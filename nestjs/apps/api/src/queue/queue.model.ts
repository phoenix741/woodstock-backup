import { Field, Int, ObjectType } from '@nestjs/graphql';

@ObjectType()
export class QueueStats {
  @Field(() => Int)
  waiting!: number;

  @Field(() => Int)
  active!: number;

  @Field(() => Int)
  failed!: number;

  @Field(() => Int)
  delayed!: number;

  @Field(() => Int)
  completed!: number;

  lastExecution?: number;
  nextWakeup?: number;
}
