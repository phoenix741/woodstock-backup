import { join } from 'path';
import { loadSync } from 'protobufjs';
import { Observable } from 'rxjs';

import { LogEntry } from '../models';
import {
  ExecuteCommandReply,
  ExecuteCommandRequest,
  FileChunk,
  GetChunkRequest,
  LaunchBackupReply,
  LaunchBackupRequest,
  RefreshCacheReply,
  RefreshCacheRequest,
} from '../models/query.model';

export interface WoodstockClientService {
  executeCommand(request: ExecuteCommandRequest): Observable<ExecuteCommandReply>;

  launchBackup(request: Observable<LaunchBackupRequest>): Observable<LaunchBackupReply>;

  refreshCache(request: Observable<RefreshCacheRequest>): Observable<RefreshCacheReply>;

  getChunk(request: GetChunkRequest): Observable<FileChunk>;

  streamLog(request: Record<string, never>): Observable<LogEntry>;
}
