import { Backup } from '@woodstock/shared';

export class HostInformation {
  constructor(public name: string, public lastBackup?: Backup) {}
}
