import { Injectable } from '@nestjs/common';
import {
  CorePoolCleanerService,
  generateContext,
  JsCleanerStatusUpdate,
  JsEventSource,
  LogContext,
} from '@woodstock/shared-rs';
import { Observable } from 'rxjs';

@Injectable()
export class CleanupMachineService {
  execute(logContext: LogContext, target?: string, abort?: AbortSignal): Observable<JsCleanerStatusUpdate> {
    const context = generateContext({
      username: undefined,
      logContext,
    });
    const service = CorePoolCleanerService.createService(context);

    return new Observable((observer) => {
      let abortMethod: () => void = () => {};
      const abortHandle = service.cleanPool(target, JsEventSource.User, (result) => {
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
