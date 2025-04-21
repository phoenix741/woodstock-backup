import { Injectable } from '@nestjs/common';
import { CorePoolCleanerService, JsCleanerStatusUpdate, JsEventSource } from '@woodstock/shared-rs';
import { Observable } from 'rxjs';

@Injectable()
export class CleanupMachineService {
  execute(target?: string, abort?: AbortSignal): Observable<JsCleanerStatusUpdate> {
    const service = new CorePoolCleanerService();

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
