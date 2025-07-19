import { CacheModuleOptions, CacheOptionsFactory } from '@nestjs/cache-manager';
import { Injectable } from '@nestjs/common';
import { ApplicationConfigService } from './application-config.service';
import { createKeyv } from '@keyv/redis';

@Injectable()
export class CacheConfigService implements CacheOptionsFactory {
  constructor(private configService: ApplicationConfigService) {}

  createCacheOptions(): CacheModuleOptions {
    return {
      // Typescript error because compare the version of cjs and mjs version ...
      stores: [createKeyv(`redis://${this.configService.redis.host}:${this.configService.redis.port}`) as any],
      ttl: this.configService.cacheTtl,
    };
  }
}
