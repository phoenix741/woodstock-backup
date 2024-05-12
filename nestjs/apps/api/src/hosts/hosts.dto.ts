import { ObjectType } from '@nestjs/graphql';
import { Backup } from '@woodstock/shared';

@ObjectType()
export class Host {
  name!: string;
}

export class HostInformation {
  constructor(
    public name: string,
    public lastBackup?: Backup,
  ) {}
}
