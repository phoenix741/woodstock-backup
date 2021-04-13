import * as Long from 'long';
import { join } from 'path';
import { loadSync } from 'protobufjs';
import { Observable } from 'rxjs';

import { FileManifest, FileManifestJournalEntry } from './object-proto.model';

const root = loadSync(join(__dirname, 'woodstock.proto'));

export const ProtoPrepareBackupRequest = root.lookupType('woodstock.PrepareBackupRequest');
export const ProtoPrepareBackupReply = root.lookupType('woodstock.PrepareBackupReply');

export const ProtoLaunchBackupRequest = root.lookupType('woodstock.LaunchBackupRequest');
export const ProtoGetChunkRequest = root.lookupType('woodstock.GetChunkRequest');
export const ProtoFileChunk = root.lookupType('woodstock.FileChunk');

export enum StatusCode {
  Ok = 0,
  Failed = 1,
}

export enum LogLevel {
  DEBUG = 0,
  INFO = 1,
  WARN = 2,
  ERROR = 3,
  FATAL = 4,
}

export interface BackupShare {
  sharePath: Buffer;
  includes: Buffer[];
  excludes: Buffer[];
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

export interface LogEntry {
  level: LogLevel;
  line: string;
}

export interface WoodstockClientService {
  executeCommand(request: ExecuteCommandRequest): Observable<ExecuteCommandReply>;

  prepareBackup(request: PrepareBackupRequest): Observable<PrepareBackupReply>;

  refreshCache(request: Observable<FileManifest>): Observable<RefreshCacheReply>;

  launchBackup(request: LaunchBackupRequest): Observable<FileManifestJournalEntry>;

  updateFileManifest(request: Observable<FileManifestJournalEntry>): Observable<UpdateManifestReply>;

  getChunk(request: GetChunkRequest): Observable<FileChunk>;

  streamLog(request: Record<string, never>): Observable<LogEntry>;
}
