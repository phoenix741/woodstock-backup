import { Injectable } from '@nestjs/common';
import { BackupsService } from '@woodstock/shared';
import { generateContext, JsBackupSaveService, JsBackupState, LogContext } from '@woodstock/shared-rs';
import { Observable } from 'rxjs';

@Injectable()
export class BackupMachineService {
  constructor(private backupsService: BackupsService) {}

  execute(
    logContext: LogContext,
    hostname: string,
    ip: string,
    backupNumber: number,
    abort?: AbortSignal,
  ): Observable<JsBackupState> {
    const context = generateContext({
      username: undefined,
      logContext,
    });
    const service = JsBackupSaveService.createService(hostname, ip, backupNumber, context);

    return new Observable((observer) => {
      let abortMethod: () => void = () => {};
      const abortHandle = service.execute((result) => {
        if (result.progress) {
          observer.next(result.progress);
        }

        if (result.error) {
          this.backupsService.invalidateBackup(hostname, backupNumber);
          observer.error(result.error);
          abort?.removeEventListener('abort', abortMethod);
        }

        if (result.complete) {
          this.backupsService.invalidateBackup(hostname, backupNumber);
          observer.complete();
          abort?.removeEventListener('abort', abortMethod);
        }
      });
      abortMethod = () => {
        abortHandle.abort();
        observer.error(new Error('Download aborted'));
      };

      abort?.addEventListener('abort', abortMethod);
    });
  }
}
