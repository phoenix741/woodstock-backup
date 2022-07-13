import * as Long from 'long';

export const HASH_ALGO = 'sha3-256';
export const CHUNK_SIZE = 1 << 22;
export const LONG_CHUNK_SIZE = Long.fromNumber(CHUNK_SIZE);

export const WORKER_TYPE = 'worker_type';

export enum WorkerType {
  api = 'api',
  console = 'console',
  backupWorker = 'backupWorker',
  scheduleWorker = 'scheduleWorker',
  statsWorker = 'statsWorker',
  client = 'client',
}
