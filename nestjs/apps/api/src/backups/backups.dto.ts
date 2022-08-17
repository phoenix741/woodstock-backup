import { ObjectType } from '@nestjs/graphql';

@ObjectType()
export class JobResponse {
  id!: string;
}
