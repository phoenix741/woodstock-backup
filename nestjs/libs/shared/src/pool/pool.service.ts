import { Injectable } from '@nestjs/common';
import { AsyncIterableX, count } from 'ix/asynciterable';
import { filter, map } from 'ix/asynciterable/operators';
import { basename } from 'path';
import { Observable } from 'rxjs';
import { ApplicationConfigService } from '../config';
import { RefCntService, ReferenceCount } from '../refcnt';
import { FileBrowserService } from '../scanner';
import { PoolUnused } from '../shared';
import { rm } from '../utils';
import { PoolChunkWrapper } from './pool-chunk-wrapper';

@Injectable()
export class PoolService {
  constructor(
    private applicationConfig: ApplicationConfigService,
    private refcntService: RefCntService,
    private fileBrowserService: FileBrowserService,
  ) {}

  isChunkExists(sha256: Buffer): Promise<boolean> {
    return PoolChunkWrapper.exists(this.applicationConfig.poolPath, sha256);
  }

  getChunk(sha256?: Buffer): PoolChunkWrapper {
    return PoolChunkWrapper.get(this.applicationConfig.poolPath, sha256);
  }

  readAllChunks(): AsyncIterableX<PoolChunkWrapper> {
    return this.fileBrowserService
      .getFilesRecursive(Buffer.from(this.applicationConfig.poolPath))(Buffer.from(''))
      .pipe(
        map((file) => basename(file.toString())),
        filter((file) => file.endsWith('-sha256.zz')),
        map((file) => Buffer.from(file.substring(0, file.length - 10), 'hex')),
        map((file) => this.getChunk(file)),
      );
  }

  async countUnusedFiles(): Promise<number> {
    const refcnt = new ReferenceCount('', '', this.applicationConfig.poolPath);
    return await count(this.refcntService.readUnused(refcnt.unusedPoolPath));
  }

  removeUnusedFiles(targetPath?: string): Observable<PoolUnused> {
    return new Observable<PoolUnused>((observable) => {
      (async () => {
        const refcnt = new ReferenceCount('', '', this.applicationConfig.poolPath);
        const unused = this.refcntService.readUnused(refcnt.unusedPoolPath);

        for await (const chunk of unused) {
          if (targetPath) {
            await this.getChunk(chunk.sha256).mv(targetPath);
          } else {
            await this.getChunk(chunk.sha256).remove();
          }
          observable.next(chunk);
        }

        await rm(refcnt.unusedPoolPath);
        observable.complete();
      })();
    });
  }
}
