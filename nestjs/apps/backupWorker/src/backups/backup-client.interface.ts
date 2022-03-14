import { LoggerService } from '@nestjs/common';
import {
  AuthenticateReply,
  ChunkInformation,
  FileManifestJournalEntry,
  LogEntry,
  RefreshCacheReply,
  RefreshCacheRequest,
  Share,
} from '@woodstock/shared';
import { AsyncIterableX } from 'ix/asynciterable';
import { Readable } from 'stream';
import { BackupsGrpcContext } from './backup-client-grpc.class';

export interface BackupClientContext {
  host: string;
  currentBackupId: number;
  logger?: LoggerService;
  abortable: AbortController[];
}

export interface BackupClientInterface {
  authenticate(context: BackupClientContext): Promise<AuthenticateReply>;
  streamLog(context: BackupClientContext): AsyncIterable<LogEntry>;
  executeCommand(context: BackupClientContext, command: string): Promise<void>;
  refreshCache(context: BackupClientContext, request: AsyncIterable<RefreshCacheRequest>): Promise<RefreshCacheReply>;
  downloadFileList(context: BackupClientContext, backupShare: Share): AsyncIterableX<FileManifestJournalEntry>;
  copyChunk(context: BackupsGrpcContext, chunk: ChunkInformation): Readable;
  close(context: BackupClientContext): void;
}
