import { Injectable } from '@nestjs/common';
import { ApplicationConfigService } from '@woodstock/core';
import Client from 'ioredis';
import type { RedlockAbortSignal } from 'redlock';
import Redlock from 'redlock';

@Injectable()
export class LockService {
  #redlock: Redlock;

  constructor(private applicationConfigService: ApplicationConfigService) {
    const redisClient = new Client(this.applicationConfigService.redis);
    this.#redlock = new Redlock([redisClient]);
  }

  using<T>(resources: string[], timeout: number, routine: (signal: RedlockAbortSignal) => Promise<T>): Promise<T> {
    return this.#redlock.using(resources, timeout, routine);
  }

  async isLocked(resources: string[]) {
    try {
      const lock = await this.#redlock.acquire(resources, 500, { retryCount: 0 });
      await this.#redlock.release(lock);
      return false;
    } catch {
      return true;
    }
  }
}
