import { Field, ID, ObjectType, registerEnumType } from '@nestjs/graphql';
import { Backup } from '@woodstock/shared';

export enum ClientType {
  None = 'none',
  Windows = 'windows',
  Linux = 'linux',
  LinuxLite = 'linux-lite',
}

@ObjectType()
export class Host {
  @Field(() => ID)
  name!: string;
}

export class HostInformation {
  constructor(
    public name: string,
    public lastBackup?: Backup,
  ) {}
}

export enum HostAvailibilityState {
  Online = 'online',
  Offline = 'offline',
  Unknown = 'unknown',
}

registerEnumType(HostAvailibilityState, { name: 'HostAvailibilityState' });
