import { Injectable } from '@nestjs/common';
import { BackupsService } from '@woodstock/shared';
import { generateContext, JsBackupRestoreService, JsRestoreState, JsShareSelection } from '@woodstock/shared-rs';
import { Observable } from 'rxjs';

@Injectable()
export class RestoreMachineService {
  constructor(private backupsService: BackupsService) {}

  async execute(
    hostname: string,
    ip: string,
    backupNumber: number,
    destinationDirectory: string,
    selections: JsShareSelection[],
    abort?: AbortSignal,
  ): Promise<Observable<JsRestoreState>> {
    const context = generateContext({
      username: undefined,
    });
    const service = await JsBackupRestoreService.createService(hostname, ip, backupNumber, context);

    return new Observable((observer) => {
      let abortMethod: () => void = () => {};
      const abortHandle = service.execute(destinationDirectory, selections, (result) => {
        if (result.state) {
          observer.next(result.state);
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
