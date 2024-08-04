import { ObjectType } from '@nestjs/graphql';
import { Backup } from '@woodstock/shared';

export enum ClientType {
  None = 'none',
  Windows = 'windows',
  Linux = 'linux',
  LinuxLite = 'linux-lite',
}

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
