/* eslint-disable */
import * as Long from "long";
import * as _m0 from "protobufjs/minimal";

export const protobufPackage = "woodstock";

/** Status Code for reply */
export enum StatusCode {
  Ok = 0,
  Failed = 1,
  Partial = 2,
}

export enum EntryType {
  /** ADD - Add a new file */
  ADD = 0,
  /** MODIFY - Modify a file */
  MODIFY = 1,
  /** REMOVE - Remove a file */
  REMOVE = 2,
}

export enum LogLevel {
  log = 0,
  error = 1,
  warn = 2,
  debug = 3,
  verbose = 4,
}

export interface Empty {
}

export interface PoolUnused {
  sha256: Buffer;
  size?: number | undefined;
  compressedSize?: number | undefined;
}

export interface PoolRefCount {
  sha256: Buffer;
  refCount: number;
  size: number;
  compressedSize: number;
}

export interface FileManifestStat {
  ownerId?: number | undefined;
  groupId?: number | undefined;
  size?: Long | undefined;
  compressedSize?: Long | undefined;
  lastRead?: Long | undefined;
  lastModified?: Long | undefined;
  created?: Long | undefined;
  mode?: number | undefined;
  dev?: Long | undefined;
  rdev?: Long | undefined;
  ino?: Long | undefined;
  nlink?: Long | undefined;
}

export interface FileManifestAcl {
  user?: string | undefined;
  group?: string | undefined;
  mask?: number | undefined;
  other?: number | undefined;
}

/** File manifest used to store the content of a file */
export interface FileManifest {
  path: Buffer;
  stats: FileManifestStat | undefined;
  xattr: { [key: string]: Buffer };
  acl: FileManifestAcl[];
  chunks: Buffer[];
  sha256?: Buffer | undefined;
  symlink?: Buffer | undefined;
}

export interface FileManifest_XattrEntry {
  key: string;
  value: Buffer;
}

/** Journal entry */
export interface FileManifestJournalEntry {
  type: EntryType;
  manifest: FileManifest | undefined;
}

/** Log entry */
export interface LogEntry {
  level: LogLevel;
  context: string;
  line: string;
}

export interface RefreshCacheHeader {
  sharePath: Buffer;
}

export interface RefreshCacheRequest {
  header?: RefreshCacheHeader | undefined;
  fileManifest?: FileManifest | undefined;
}

export interface RefreshCacheReply {
  code: StatusCode;
  message?: string | undefined;
}

export interface LaunchBackupRequest {
  share: Share | undefined;
}

export interface LaunchBackupResponse {
  code: StatusCode;
  message?: string | undefined;
}

export interface LaunchBackupReply {
  entry: FileManifestJournalEntry | undefined;
  response: LaunchBackupResponse | undefined;
}

export interface ChunkInformation {
  filename: Buffer;
  position: Long;
  size: number;
  sha256?: Buffer | undefined;
}

export interface FileChunk {
  data: Buffer;
}

export interface ChunkResult {
  code: StatusCode;
}

export interface GetChunkRequest {
  chunk: ChunkInformation | undefined;
}

export interface GetChunkReply {
  chunk?: ChunkInformation | undefined;
  result?: ChunkResult | undefined;
  data?: FileChunk | undefined;
}

export interface StreamLogRequest {
}

export interface ExecuteCommandRequest {
  command: string;
}

export interface ExecuteCommandReply {
  code: number;
  stdout: string;
  stderr: string;
}

export interface Share {
  sharePath: Buffer;
  includes: Buffer[];
  excludes: Buffer[];
}

export interface AuthenticateRequest {
  version: number;
  token: string;
}

export interface AuthenticateReply {
  code: StatusCode;
  message?: string | undefined;
  sessionId?: string | undefined;
}

if (_m0.util.Long !== Long) {
  _m0.util.Long = Long as any;
  _m0.configure();
}
