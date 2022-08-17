import { Backup } from '@woodstock/shared';
import { ObjectType } from '@nestjs/graphql';

@ObjectType()
export class Host {
  name!: string;
}

export class HostInformation {
  constructor(public name: string, public lastBackup?: Backup) {}
}
