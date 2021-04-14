import * as Long from 'long';

import { BackupShare } from './configuration.model';

export enum StatusCode {
  Ok = 0,
  Failed = 1,
}

export interface PrepareBackupRequest {
  configuration: BackupShare;
  lastBackupNumber: number;
  newBackupNumber: number;
}

export interface PrepareBackupReply {
  code: StatusCode;
  needRefreshCache: boolean;
}

export interface LaunchBackupRequest {
  configuration: BackupShare;
  lastBackupNumber: number;
  newBackupNumber: number;
}

export interface GetChunkRequest {
  filename: Buffer;
  position: Long;
  size: Long;
  sha256: Buffer;
}

export interface FileChunk {
  data: Buffer;
}

export interface ExecuteCommandRequest {
  command: string;
}

export interface ExecuteCommandReply {
  code: number;
  stdout: string;
  stderr: string;
}

export interface RefreshCacheReply {
  code: StatusCode;
}

export interface UpdateManifestReply {
  code: StatusCode;
}
