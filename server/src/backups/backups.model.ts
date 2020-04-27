import { ObjectType } from '@nestjs/graphql';
import { JobId } from 'bull';

@ObjectType()
export class JobResponse {
  id!: number;
}
