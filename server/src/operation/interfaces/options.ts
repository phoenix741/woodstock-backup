import { BackupLogger } from '../../logger/BackupLogger.logger';
export class BackupProgression {
  newFileSize = 0;
  fileSize = 0;
  newFileCount = 0;
  fileCount = 0;
  speed = 0;

  constructor(public percent = 0) {}
}

export type CallbackProgressFn = (progression: BackupProgression) => void;

export interface Options {
  host: string;
  context: string;

  backupLogger: BackupLogger;
  callbackProgress: CallbackProgressFn;
}
