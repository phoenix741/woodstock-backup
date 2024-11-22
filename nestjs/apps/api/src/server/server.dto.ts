import { Field, ObjectType } from '@nestjs/graphql';

@ObjectType()
export class ServerInformations {
  hostname!: string;
  @Field(() => String)
  platform!: NodeJS.Platform;
  uptime!: number;
  woodstockVersion?: string;
}
