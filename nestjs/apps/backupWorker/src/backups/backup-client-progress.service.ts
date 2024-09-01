import { Injectable } from '@nestjs/common';
import { JsBackupProgression, WoodstockBackupClient, WoodstockBackupShare } from '@woodstock/shared-rs';
import { QueueTaskProgression } from '@woodstock/shared';
import { defer, endWith, map, Observable, startWith } from 'rxjs';
import { BackupsClientService } from './backups-client.service';

function toProgress(value: JsBackupProgression): QueueTaskProgression {
  return new QueueTaskProgression({
    startDate: value.startDate,
    startTransferDate: value.startTransferDate,
    endTransferDate: value.endTransferDate,

    compressedFileSize: value.compressedFileSize,
    newCompressedFileSize: value.newCompressedFileSize,
    modifiedCompressedFileSize: value.modifiedCompressedFileSize,

    fileSize: value.fileSize,
    newFileSize: value.newFileSize,
    modifiedFileSize: value.modifiedFileSize,

    fileCount: value.fileCount,
    newFileCount: value.newFileCount,
    modifiedFileCount: value.modifiedFileCount,
    removedFileCount: value.removedFileCount,

    errorCount: value.errorCount,

    progressCurrent: value.progressCurrent,
    progressMax: value.progressMax,
  });
}

@Injectable()
export class BackupClientProgress {
  constructor(private backupClient: BackupsClientService) {}

  async createClient(hostname: string, ip: string, backupNumber: number): Promise<WoodstockBackupClient> {
    return this.backupClient.createClient(hostname, ip, backupNumber);
  }

  authenticate(context: WoodstockBackupClient, password: string): Promise<void> {
    return this.backupClient.authenticate(context, password);
  }

  createBackupDirectory(context: WoodstockBackupClient, shares: Array<string>): Observable<QueueTaskProgression> {
    const executeCommand$ = defer(() => this.backupClient.createBackupDirectory(context, shares));

    return executeCommand$.pipe(
      startWith(new QueueTaskProgression({ progressCurrent: 0n, progressMax: 0n })),
      map(() => new QueueTaskProgression()),
      endWith(new QueueTaskProgression({ progressCurrent: 1n, progressMax: 1n })),
    );
  }

  executeCommand(context: WoodstockBackupClient, command: string): Observable<QueueTaskProgression> {
    const executeCommand$ = defer(() => this.backupClient.executeCommand(context, command));

    return executeCommand$.pipe(
      startWith(new QueueTaskProgression({ progressCurrent: 0n, progressMax: 0n })),
      map(() => new QueueTaskProgression()),
      endWith(new QueueTaskProgression({ progressCurrent: 1n, progressMax: 1n })),
    );
  }

  synchronizeFileList(context: WoodstockBackupClient, share: WoodstockBackupShare): Observable<QueueTaskProgression> {
    return this.backupClient.synchronizeFileList(context, share).pipe(
      map((value) => {
        const progression = toProgress(value);
        return new QueueTaskProgression({
          startDate: progression.startDate,
          startTransferDate: progression.startTransferDate,
          endTransferDate: progression.endTransferDate,
          progressMax: progression.progressMax,
        });
      }),
    );
  }

  createBackup(context: WoodstockBackupClient, sharePath: string): Observable<QueueTaskProgression> {
    return this.backupClient.createBackup(context, sharePath).pipe(
      map((value) => {
        const progression = toProgress(value);
        return new QueueTaskProgression({
          startDate: progression.startDate,
          startTransferDate: progression.startTransferDate,
          endTransferDate: progression.endTransferDate,
          progressCurrent: progression.progressCurrent,
          fileCount: progression.fileCount,
          newFileCount: progression.newFileCount,
          modifiedFileCount: progression.modifiedFileCount,
          removedFileCount: progression.removedFileCount,
          newFileSize: progression.newFileSize,
          modifiedFileSize: progression.modifiedFileSize,
          newCompressedFileSize: progression.newCompressedFileSize,
          modifiedCompressedFileSize: progression.modifiedCompressedFileSize,
          errorCount: progression.errorCount,
        });
      }),
    );
  }

  compact(context: WoodstockBackupClient, sharePath: string): Observable<QueueTaskProgression> {
    const compact$ = defer(() => this.backupClient.compact(context, sharePath));

    return compact$.pipe(
      startWith(new QueueTaskProgression({ progressCurrent: 0n, progressMax: 0n })),
      map(() => new QueueTaskProgression()),
      endWith(new QueueTaskProgression({ progressCurrent: 1n, progressMax: 1n })),
    );
  }

  countReferences(context: WoodstockBackupClient): Observable<QueueTaskProgression> {
    const countRef$ = defer(() => this.backupClient.countReferences(context));

    return countRef$.pipe(
      startWith(new QueueTaskProgression({ progressCurrent: 0n, progressMax: 0n })),
      map(() => new QueueTaskProgression()),
      startWith(new QueueTaskProgression({ progressCurrent: 1n, progressMax: 1n })),
    );
  }

  saveBackup(context: WoodstockBackupClient, finished: boolean, completed: boolean): Observable<QueueTaskProgression> {
    const saveBackup$ = defer(() => this.backupClient.saveBackup(context, finished, completed));

    return saveBackup$.pipe(
      startWith(new QueueTaskProgression({ progressCurrent: 0n, progressMax: 0n })),
      map(() => new QueueTaskProgression()),
      startWith(new QueueTaskProgression({ progressCurrent: 1n, progressMax: 1n })),
    );
  }

  close(context: WoodstockBackupClient): Promise<void> {
    return this.backupClient.close(context);
  }
}
