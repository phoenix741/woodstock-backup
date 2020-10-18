import { Backup } from '../backups/backup.dto';

export class HostInformation {
  constructor(public name: string, public lastBackup?: Backup) {}
}
