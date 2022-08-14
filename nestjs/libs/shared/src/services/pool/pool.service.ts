import { Injectable } from '@nestjs/common';
import { PoolChunkInformation } from '@woodstock/shared/models';
import { Observable } from 'rxjs';
import { ApplicationConfigService } from '../../config';
import { RefCntService, ReferenceCount } from '../../refcnt';
import { rm } from '../../utils';
import { PoolChunkWrapper } from './pool-chunk-wrapper';

@Injectable()
export class PoolService {
  constructor(private applicationConfig: ApplicationConfigService, private refcntService: RefCntService) {}

  isChunkExists(sha256: Buffer): Promise<boolean> {
    return PoolChunkWrapper.exists(this, this.applicationConfig.poolPath, sha256);
  }

  getChunk(sha256?: Buffer): PoolChunkWrapper {
    return PoolChunkWrapper.get(this, this.applicationConfig.poolPath, sha256);
  }

  removeUnusedFiles(targetPath?: string): Observable<PoolChunkInformation> {
    return new Observable<PoolChunkInformation>((observable) => {
      (async () => {
        const refcnt = new ReferenceCount('', '', this.applicationConfig.poolPath);
        const unused = this.refcntService.readUnused(refcnt.unusedPoolPath);

        for await (const chunk of unused) {
          let removedChunk: PoolChunkInformation;
          if (targetPath) {
            removedChunk = await PoolChunkWrapper.get(this, this.applicationConfig.poolPath, chunk.sha256).mv(
              targetPath,
            );
          } else {
            removedChunk = await PoolChunkWrapper.get(this, this.applicationConfig.poolPath, chunk.sha256).remove();
          }
          observable.next(removedChunk);
        }

        await rm(refcnt.unusedPoolPath);
        observable.complete();
      })();
    });
  }
}
