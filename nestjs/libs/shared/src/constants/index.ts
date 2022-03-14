import * as Long from 'long';

export const HASH_ALGO = 'sha3-256';
export const CHUNK_SIZE = 1 << 22;
export const LONG_CHUNK_SIZE = Long.fromNumber(CHUNK_SIZE);
