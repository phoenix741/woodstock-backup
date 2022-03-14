import { Backup } from '@woodstock/backoffice-shared';

export class HostInformation {
  constructor(public name: string, public lastBackup?: Backup) {}
}
