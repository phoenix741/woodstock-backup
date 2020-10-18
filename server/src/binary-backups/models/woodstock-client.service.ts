import { BackupConfiguration } from './backups-configuration.model';
import { Observable } from 'rxjs';
import { FileManifestJournalEntry } from '../../storage/backup-manifest/manifest.model';
import { Readable } from 'stream';

export enum StatusCode {
  Ok = 0,
  Failed = 1,
}

export interface PrepareBackupRequest {
  configuration: BackupConfiguration;
  lastBackupNumber: number;
  newBackupNumber: number;
}

export interface PrepareBackupReply {
  code: StatusCode;
  needRefreshCache: boolean;
}

export interface LaunchBackupRequest {
  backupNumber: number;
}

export interface GetChunkRequest {
  filename: Buffer;
  position: number;
  size: number;
  sha256: Buffer;
}

export interface FileChunk {
  data: Buffer;
}

export interface WoodstockClientService {
  prepareBackup(request: PrepareBackupRequest): Observable<PrepareBackupReply>;

  launchBackup(request: LaunchBackupRequest): Observable<FileManifestJournalEntry>;

  getChunk(request: GetChunkRequest): Observable<FileChunk>;
  //getChunk(request: GetChunkRequest): Observable<Readable>;
}
