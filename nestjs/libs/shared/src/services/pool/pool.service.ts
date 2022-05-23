import { Injectable } from '@nestjs/common';
import { ApplicationConfigService } from '../../config';
import { YamlService } from '../yaml.service';
import { PoolChunkWrapper } from './pool-chunk-wrapper';

@Injectable()
export class PoolService {
  constructor(private applicationConfig: ApplicationConfigService, private yamlService: YamlService) {}

  isChunkExists(sha256: Buffer): Promise<boolean> {
    return PoolChunkWrapper.exists(this, this.applicationConfig.poolPath, sha256);
  }

  getChunk(sha256?: Buffer): PoolChunkWrapper {
    return PoolChunkWrapper.get(this, this.applicationConfig.poolPath, sha256);
  }

  // async incrStatistics(info: PoolChunkInformation): Promise<void> {
  //   const unlock = await lock(this.poolStatisticsFileName, { realpath: false });
  //   try {
  //     const poolStatistics = await this.readPoolStatistics();
  //     poolStatistics.fileCount++;
  //     poolStatistics.poolSize += info.size;
  //     poolStatistics.compressedPoolSize += info.compressedSize;
  //     await this.writePoolStatistics(poolStatistics);
  //   } finally {
  //     await unlock();
  //   }
  // }

  // async decrStatistics(info: PoolChunkInformation): Promise<void> {
  //   const unlock = await lock(this.poolStatisticsFileName, { realpath: false });
  //   try {
  //     const poolStatistics = await this.readPoolStatistics();
  //     poolStatistics.fileCount--;
  //     poolStatistics.poolSize -= info.size;
  //     poolStatistics.compressedPoolSize -= info.compressedSize;
  //     await this.writePoolStatistics(poolStatistics);
  //   } finally {
  //     await unlock();
  //   }
  // }

  // private get poolStatisticsFileName() {
  //   return join(this.applicationConfig.poolPath, 'pool.statistics');
  // }

  // private async readPoolStatistics(): Promise<PoolSize> {
  //   return await this.yamlService.loadFile<PoolSize>(this.poolStatisticsFileName, {
  //     fileCount: 0n,
  //     poolSize: 0n,
  //     compressedPoolSize: 0n,
  //   } as PoolSize);
  // }

  // private async writePoolStatistics(poolSize: PoolSize): Promise<void> {
  //   await this.yamlService.writeFile(this.poolStatisticsFileName, poolSize);
  // }
}
