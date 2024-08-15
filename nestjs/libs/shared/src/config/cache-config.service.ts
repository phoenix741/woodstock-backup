import { CacheOptions, CacheOptionsFactory } from '@nestjs/cache-manager';
import { Injectable } from '@nestjs/common';
import { ApplicationConfigService } from './application-config.service';
import { redisStore } from 'cache-manager-ioredis-yet';
import { RedisOptions } from 'ioredis';

@Injectable()
export class CacheConfigService implements CacheOptionsFactory<RedisOptions> {
  constructor(private configService: ApplicationConfigService) {}

  createCacheOptions(): CacheOptions<RedisOptions> | Promise<CacheOptions<RedisOptions>> {
    return {
      store: redisStore,

      host: this.configService.redis.host,
      port: this.configService.redis.port,

      ttl: this.configService.cacheTtl,
    };
  }
}
