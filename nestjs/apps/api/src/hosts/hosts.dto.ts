import { ObjectType } from '@nestjs/graphql';
import { Backup } from '@woodstock/server';

@ObjectType()
export class Host {
  name!: string;
}

export class HostInformation {
  constructor(public name: string, public lastBackup?: Backup) {}
}
