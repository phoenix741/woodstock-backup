import { ChunkInformation } from '@woodstock/shared';

export interface PoolChunkInformation {
  originalInformation?: ChunkInformation;
  sha256: Buffer;
  size: bigint;
  compressedSize: bigint;
}

export interface PoolChunkError {
  originalInformation?: ChunkInformation;
  error: any;
}

export function isPoolChunkError(value: PoolChunkInformation | PoolChunkError): value is PoolChunkError {
  return (value as PoolChunkError).error !== undefined;
}

export function isPoolChunkInformation(value: PoolChunkInformation | unknown): value is PoolChunkInformation {
  return (value as PoolChunkInformation).sha256 !== undefined;
}
