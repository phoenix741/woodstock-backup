import { join } from 'path';
import { loadSync } from 'protobufjs';
import { Observable } from 'rxjs';
import { FileManifest, FileManifestJournalEntry, LogEntry } from '../models';

import {
  ExecuteCommandReply,
  ExecuteCommandRequest,
  FileChunk,
  GetChunkRequest,
  LaunchBackupRequest,
  PrepareBackupReply,
  PrepareBackupRequest,
  RefreshCacheReply,
  UpdateManifestReply,
} from '../models/query.model';

const root = loadSync(join(__dirname, '..', '..', 'woodstock.proto'));

export const ProtoPrepareBackupRequest = root.lookupType('woodstock.PrepareBackupRequest');
export const ProtoPrepareBackupReply = root.lookupType('woodstock.PrepareBackupReply');

export const ProtoLaunchBackupRequest = root.lookupType('woodstock.LaunchBackupRequest');
export const ProtoGetChunkRequest = root.lookupType('woodstock.GetChunkRequest');
export const ProtoFileChunk = root.lookupType('woodstock.FileChunk');

export interface WoodstockClientService {
  executeCommand(request: ExecuteCommandRequest): Observable<ExecuteCommandReply>;

  prepareBackup(request: PrepareBackupRequest): Observable<PrepareBackupReply>;

  refreshCache(request: Observable<FileManifest>): Observable<RefreshCacheReply>;

  launchBackup(request: LaunchBackupRequest): Observable<FileManifestJournalEntry>;

  updateFileManifest(request: Observable<FileManifestJournalEntry>): Observable<UpdateManifestReply>;

  getChunk(request: GetChunkRequest): Observable<FileChunk>;

  streamLog(request: Record<string, never>): Observable<LogEntry>;
}
