import { Injectable } from '@nestjs/common';
import { PoolUnused } from '@woodstock/shared/models';
import { AsyncIterableX } from 'ix/asynciterable';
import { filter, map } from 'ix/asynciterable/operators';
import { basename } from 'path';
import { Observable } from 'rxjs';
import { ApplicationConfigService } from '../../config';
import { FileBrowserService } from '../../file/file-browser.service';
import { PoolChunkInformation } from '../../models/pool-chunk.dto';
import { RefCntService, ReferenceCount } from '../../refcnt';
import { rm } from '../../utils';
import { PoolChunkWrapper } from './pool-chunk-wrapper';

@Injectable()
export class PoolService {
  constructor(
    private applicationConfig: ApplicationConfigService,
    private refcntService: RefCntService,
    private fileBrowserService: FileBrowserService,
  ) {}

  isChunkExists(sha256: Buffer): Promise<boolean> {
    return PoolChunkWrapper.exists(this, this.applicationConfig.poolPath, sha256);
  }

  getChunk(sha256?: Buffer): PoolChunkWrapper {
    return PoolChunkWrapper.get(this, this.applicationConfig.poolPath, sha256);
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

  removeUnusedFiles(targetPath?: string): Observable<PoolUnused> {
    return new Observable<PoolUnused>((observable) => {
      (async () => {
        const refcnt = new ReferenceCount('', '', this.applicationConfig.poolPath);
        const unused = this.refcntService.readUnused(refcnt.unusedPoolPath);

        for await (const chunk of unused) {
          if (targetPath) {
            await PoolChunkWrapper.get(this, this.applicationConfig.poolPath, chunk.sha256).mv(targetPath);
          } else {
            await PoolChunkWrapper.get(this, this.applicationConfig.poolPath, chunk.sha256).remove();
          }
          observable.next(chunk);
        }

        await rm(refcnt.unusedPoolPath);
        observable.complete();
      })();
    });
  }
}
