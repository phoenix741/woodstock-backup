import * as Long from 'long';

import { FileManifest, FileManifestJournalEntry } from './manifest.model';

export enum StatusCode {
  Ok = 0,
  Failed = 1,
  Partial = 2,
}

export interface FileChunk {
  data: Buffer;
}

export interface RefreshCacheHeader {
  sharePath: Buffer;
}

export interface RefreshCacheRequest {
  header?: RefreshCacheHeader;
  manifest?: FileManifest;
}

export interface RefreshCacheReply {
  code: StatusCode;
  message?: string;
}

export interface LaunchBackupHeader {
  sharePath: Buffer;
  includes?: Buffer[];
  excludes?: Buffer[];
  lastBackupNumber: number;
  newBackupNumber: number;
}

export interface LaunchBackupFooter {
  code: StatusCode;
  message?: string;
}

export interface LaunchBackupRequest {
  header?: LaunchBackupHeader;
  entry?: FileManifestJournalEntry;
  footer?: LaunchBackupFooter;
}

export interface LaunchBackupResponse {
  code: StatusCode;
  message?: string;
  needRefreshCache?: boolean;
  diskReadFinished?: boolean;
}

export interface LaunchBackupReply {
  entry?: FileManifestJournalEntry;
  response?: LaunchBackupResponse;
}

export interface GetChunkRequest {
  filename: Buffer;
  position: Long;
  size: Long;
  sha256: Buffer;
  failIfWrongHash?: boolean;
}

export interface ExecuteCommandRequest {
  command: string;
}

export interface ExecuteCommandReply {
  code: number;
  stdout: string;
  stderr: string;
}
