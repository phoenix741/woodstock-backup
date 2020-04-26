import { Injectable } from '@nestjs/common';
import { JobId } from 'bull';
import * as fs from 'fs';
import * as mkdirp from 'mkdirp';
import { dirname } from 'path';

@Injectable()
export class LockService {
  constructor() {}

  async lock(lockfile: string, jobId: JobId, force = false): Promise<JobId | null> {
    try {
      await mkdirp(dirname(lockfile));

      await fs.promises.writeFile(lockfile, JSON.stringify(jobId), {
        encoding: 'utf-8',
        flag: force ? 'w' : 'wx',
      });

      return null;
    } catch (err) {
      if (err.code === 'EEXIST') {
        const previousLock = JSON.parse(await fs.promises.readFile(lockfile, 'utf-8'));
        if (previousLock === jobId) {
          return await this.lock(lockfile, jobId, true);
        }
        return previousLock;
      }

      throw err;
    }
  }

  async isLocked(lockfile: string): Promise<JobId | null> {
    try {
      return JSON.parse(await fs.promises.readFile(lockfile, 'utf-8'));
    } catch (err) {
      if (err.code === 'ENOENT') {
        // Not locked
        return null;
      }
      throw err;
    }
  }

  async unlock(lockfile: string, jobId: JobId, force = false): Promise<JobId | null> {
    try {
      const currentLock = JSON.parse(await fs.promises.readFile(lockfile, 'utf-8'));

      if (currentLock !== jobId && !force) {
        return currentLock;
      }

      await fs.promises.unlink(lockfile);
      return null;
    } catch (err) {
      if (err.code === 'ENOENT') {
        // Not locked
        return null;
      }
      throw err;
    }
  }
}
