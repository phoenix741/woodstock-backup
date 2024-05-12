import { Injectable } from '@nestjs/common';
import {
  CorePoolService,
  JsFsckCount,
  JsFsckUnusedCount,
  JsPoolUnused,
  VerifyChunkCount,
  VerifyChunkProgress,
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

  countUnused(): Promise<bigint> {
    return this.lockService.using([POOL_RESOURCE_LOCK], REFCNT_LOCK_TIMEOUT, () => this.poolService.countUnused());
  }

  removeUnused(target?: string): Observable<JsPoolUnused> {
    let removeUnused = this.lockService.using([POOL_RESOURCE_LOCK], REFCNT_LOCK_TIMEOUT, async (abort) => {
      return new Observable<JsPoolUnused>((observer) => {
        let abortMethod: () => void = () => {};
        let abortHandle = this.poolService.removeUnused(target, (result) => {
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

  countChunk(): Promise<bigint> {
    return this.lockService.using([POOL_RESOURCE_LOCK], REFCNT_LOCK_TIMEOUT, () => this.poolService.countChunk());
  }

  verifyChunk(): Observable<VerifyChunkCount> {
    let verifyChunk = this.lockService.using([POOL_RESOURCE_LOCK], REFCNT_LOCK_TIMEOUT, async (abort) => {
      return new Observable<VerifyChunkCount>((observer) => {
        let abortMethod: () => void = () => {};
        let abortHandle = this.poolService.verifyChunk((result) => {
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

  checkBackupIntegrity(hostname: string, backupNumber: number, dryRun: boolean): Promise<JsFsckCount> {
    return this.lockService.using([POOL_RESOURCE_LOCK], REFCNT_LOCK_TIMEOUT, () =>
      this.poolService.checkBackupIntegrity(hostname, backupNumber, dryRun),
    );
  }

  checkHostIntegrity(hostname: string, dryRun: boolean): Promise<JsFsckCount> {
    return this.lockService.using([POOL_RESOURCE_LOCK], REFCNT_LOCK_TIMEOUT, () =>
      this.poolService.checkHostIntegrity(hostname, dryRun),
    );
  }

  checkPoolIntegrity(dryRun: boolean): Promise<JsFsckCount> {
    return this.lockService.using([POOL_RESOURCE_LOCK], REFCNT_LOCK_TIMEOUT, () =>
      this.poolService.checkPoolIntegrity(dryRun),
    );
  }

  processUnused(dryRun: boolean): Observable<VerifyChunkProgress> {
    let verifyChunk = this.lockService.using([POOL_RESOURCE_LOCK], REFCNT_LOCK_TIMEOUT, async (abort) => {
      return new Observable<VerifyChunkProgress>((observer) => {
        let abortMethod: () => void = () => {};
        let abortHandle = this.poolService.checkUnused(dryRun, (result) => {
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
