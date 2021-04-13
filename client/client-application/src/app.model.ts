import * as Long from 'long';

export enum EntryType {
  ADD = 0,
  MODIFY = 1,
  REMOVE = 2,
  CLOSE = 255,
}

export interface FileManifestStat {
  ownerId?: Long;
  groupId?: Long;
  size?: Long;
  lastRead?: Long;
  lastModified?: Long;
  created?: Long;
  mode?: Long;
}

export interface FileManifestAcl {
  user?: string;
  group?: string;
  mask?: number;
  other?: number;
}

export interface FileManifest {
  path: Buffer;
  stats?: FileManifestStat;
  xattr?: Record<string, Buffer>;
  acl?: FileManifestAcl[];
  chunks?: Buffer[];
  sha256?: Buffer;
}

export interface FileManifestJournalEntryRemove {
  type: EntryType.REMOVE;
  path: Buffer;
}

export interface FileManifestJournalEntryAddOrModify {
  type: EntryType.ADD | EntryType.MODIFY;
  manifest: FileManifest;
}

export interface FileManifestJournalEntryClose {
  type: EntryType.CLOSE;
}

export type FileManifestJournalEntry =
  | FileManifestJournalEntryRemove
  | FileManifestJournalEntryAddOrModify
  | FileManifestJournalEntryClose;

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
