import * as Long from 'long';
import { join } from 'path';
import { loadSync } from 'protobufjs';
import { Observable } from 'rxjs';

import { BackupConfiguration } from '../../binary-backups/models/backups-configuration.model';
import { FileManifestJournalEntry } from './object-proto.model';

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
  position: Long;
  size: Long;
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
