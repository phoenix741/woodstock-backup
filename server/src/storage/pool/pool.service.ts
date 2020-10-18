import { Injectable } from '@nestjs/common';

import { ApplicationConfigService } from '../../config/application-config.service';
import { PoolChunkWrapper } from './pool-chunk-wrapper';

@Injectable()
export class PoolService {
  constructor(private applicationConfig: ApplicationConfigService) {}

  isChunkExists(sha256: Buffer): Promise<boolean> {
    return PoolChunkWrapper.exists(this.applicationConfig.poolPath, sha256);
  }

  getChunk(sha256: Buffer): PoolChunkWrapper {
    return PoolChunkWrapper.get(this.applicationConfig.poolPath, sha256);
  }
}
