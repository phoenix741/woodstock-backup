import { ObjectType } from '@nestjs/graphql';

@ObjectType()
export class Host {
  name!: string;
}
