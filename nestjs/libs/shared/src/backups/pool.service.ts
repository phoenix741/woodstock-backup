import { Injectable } from '@nestjs/common';
import {
  CorePoolService,
  FsckProgressMessage,
  JsFsckCount,
  JsFsckProgression,
  JsPoolProgression,
} from '@woodstock/shared-rs';
import { defer, Observable, switchMap } from 'rxjs';
import { LockService } from './lock.service';

const POOL_RESOURCE_LOCK = 'pool';
const REFCNT_LOCK_TIMEOUT = 60 * 1000;

@Injectable()
export class PoolService {
  constructor(
    private poolService: CorePoolService,
    private lockService: LockService,
  ) {}

  addRefcntOfPool(hostname: string, backupNumber: number): Promise<void> {
    return this.lockService.using([POOL_RESOURCE_LOCK], REFCNT_LOCK_TIMEOUT, () =>
      this.poolService.addRefcntOfPool(hostname, backupNumber),
    );
  }

  removeRefcntOfPool(hostname: string, backupNumber: number): Promise<void> {
    return this.lockService.using([POOL_RESOURCE_LOCK], REFCNT_LOCK_TIMEOUT, () =>
      this.poolService.removeRefcntOfPool(hostname, backupNumber),
    );
  }

  countUnused(): Promise<number> {
    return this.lockService.using([POOL_RESOURCE_LOCK], REFCNT_LOCK_TIMEOUT, () => this.poolService.removeUnusedMax());
  }

  removeUnused(target?: string): Observable<JsPoolProgression> {
    const removeUnused = this.lockService.using([POOL_RESOURCE_LOCK], REFCNT_LOCK_TIMEOUT, async (abort) => {
      return new Observable<JsPoolProgression>((observer) => {
        let abortMethod: () => void = () => {};
        const abortHandle = this.poolService.removeUnused(target, (result) => {
          if (result.progress) {
            observer.next(result.progress);
          }

          if (result.error) {
            abort?.removeEventListener('abort', abortMethod);
            observer.error(result.error);
          }

          if (result.complete) {
            abort?.removeEventListener('abort', abortMethod);
            observer.complete();
          }
        });
        abortMethod = () => {
          abortHandle.abort();
        };

        abort?.addEventListener('abort', abortMethod);
      });
    });
    return defer(() => removeUnused).pipe(switchMap((x) => x));
  }

  countChunk(): Promise<number> {
    return this.lockService.using([POOL_RESOURCE_LOCK], REFCNT_LOCK_TIMEOUT, () => this.poolService.verifyChunkMax());
  }

  verifyChunk(): Observable<JsFsckProgression> {
    const verifyChunk = this.lockService.using([POOL_RESOURCE_LOCK], REFCNT_LOCK_TIMEOUT, async (abort) => {
      return new Observable<JsFsckProgression>((observer) => {
        let abortMethod: () => void = () => {};
        const abortHandle = this.poolService.verifyChunk((result) => {
          if (result.progress) {
            observer.next(result.progress);
          }

          if (result.error) {
            abort?.removeEventListener('abort', abortMethod);
            observer.error(result.error);
          }

          if (result.complete) {
            abort?.removeEventListener('abort', abortMethod);
            observer.complete();
          }
        });
        abortMethod = () => {
          abortHandle.abort();
        };

        abort?.addEventListener('abort', abortMethod);
      });
    });
    return defer(() => verifyChunk).pipe(switchMap((x) => x));
  }

  verifyRefcntMax(): Promise<number> {
    return this.lockService.using([POOL_RESOURCE_LOCK], REFCNT_LOCK_TIMEOUT, () => this.poolService.verifyUnusedMax());
  }

  verifyRefcnt(dryRun: boolean): Observable<JsPoolProgression> {
    const verifyChunk = this.lockService.using([POOL_RESOURCE_LOCK], REFCNT_LOCK_TIMEOUT, async (abort) => {
      return new Observable<JsPoolProgression>((observer) => {
        let abortMethod: () => void = () => {};
        const abortHandle = this.poolService.verifyUnused(dryRun, (result) => {
          if (result.progress) {
            observer.next(result.progress);
          }

          if (result.error) {
            abort?.removeEventListener('abort', abortMethod);
            observer.error(result.error);
          }

          if (result.complete) {
            abort?.removeEventListener('abort', abortMethod);
            observer.complete();
          }
        });
        abortMethod = () => {
          abortHandle.abort();
        };

        abort?.addEventListener('abort', abortMethod);
      });
    });
    return defer(() => verifyChunk).pipe(switchMap((x) => x));
  }

  verifyUnusedMax(): Promise<number> {
    return this.lockService.using([POOL_RESOURCE_LOCK], REFCNT_LOCK_TIMEOUT, () => this.poolService.verifyUnusedMax());
  }

  verifyUnused(dryRun: boolean): Observable<JsPoolProgression> {
    const verifyChunk = this.lockService.using([POOL_RESOURCE_LOCK], REFCNT_LOCK_TIMEOUT, async (abort) => {
      return new Observable<JsPoolProgression>((observer) => {
        let abortMethod: () => void = () => {};
        const abortHandle = this.poolService.verifyUnused(dryRun, (result) => {
          if (result.progress) {
            observer.next(result.progress);
          }

          if (result.error) {
            abort?.removeEventListener('abort', abortMethod);
            observer.error(result.error);
          }

          if (result.complete) {
            abort?.removeEventListener('abort', abortMethod);
            observer.complete();
          }
        });
        abortMethod = () => {
          abortHandle.abort();
        };

        abort?.addEventListener('abort', abortMethod);
      });
    });
    return defer(() => verifyChunk).pipe(switchMap((x) => x));
  }
}
