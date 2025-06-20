import { Injectable } from '@nestjs/common';
import {
  CorePoolFsckService,
  generateContext,
  JsEventSource,
  JsFsckStatusUpdate,
  LogContext,
} from '@woodstock/shared-rs';
import { Observable } from 'rxjs';

@Injectable()
export class FsckMachineService {
  execute(
    logContext: LogContext,
    dryRun: boolean,
    verifyChunks: boolean,
    abort?: AbortSignal,
  ): Observable<JsFsckStatusUpdate> {
    const context = generateContext({
      username: undefined,
      logContext,
    });
    const service = CorePoolFsckService.createService(context);

    return new Observable((observer) => {
      let abortMethod: () => void = () => {};
      const abortHandle = service.executeFsck(dryRun, verifyChunks, JsEventSource.User, (result) => {
        if (result.progress) {
          observer.next(result.progress);
        }

        if (result.error) {
          observer.error(result.error);
          abort?.removeEventListener('abort', abortMethod);
        }

        if (result.complete) {
          observer.complete();
          abort?.removeEventListener('abort', abortMethod);
        }
      });
      abortMethod = () => {
        abortHandle.abort();
        observer.error(new Error('Fsck aborted'));
      };

      abort?.addEventListener('abort', abortMethod);
    });
  }
}
