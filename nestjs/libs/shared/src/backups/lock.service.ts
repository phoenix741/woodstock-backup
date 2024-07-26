import { Injectable } from '@nestjs/common';
import Client from 'ioredis';
import Redlock from 'redlock';

import { ApplicationConfigService } from '../config';

@Injectable()
export class LockService {
  #redlock: Redlock;

  constructor(private applicationConfigService: ApplicationConfigService) {
    const redisClient = new Client(this.applicationConfigService.redis);
    this.#redlock = new Redlock([redisClient]);
  }

  using<T>(resources: string[], timeout: number, routine: (signal: AbortSignal) => Promise<T>): Promise<T> {
    return this.#redlock.using(resources, timeout, routine);
  }

  async isLocked(resources: string[]) {
    try {
      const lock = await this.#redlock.acquire(resources, 500, {
        retryCount: 0,
        retryJitter: 200,
        retryDelay: 200,
        driftFactor: 0.01,
        automaticExtensionThreshold: 500,
      });
      await this.#redlock.release(lock);
      return false;
    } catch {
      return true;
    }
  }
}
