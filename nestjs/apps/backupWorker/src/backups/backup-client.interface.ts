import { BackupClientContext } from '@woodstock/server';
import {
  AuthenticateReply,
  ChunkInformation,
  ExecuteCommandReply,
  FileManifestJournalEntry,
  LogEntry,
  RefreshCacheReply,
  RefreshCacheRequest,
  Share,
} from '@woodstock/shared';
import { AsyncIterableX } from 'ix/asynciterable';
import { Readable } from 'stream';

export interface BackupClientInterface {
  createConnection(context: BackupClientContext): Promise<void>;
  authenticate(context: BackupClientContext, password: string): Promise<AuthenticateReply>;
  streamLog(context: BackupClientContext): AsyncIterableX<LogEntry>;
  executeCommand(context: BackupClientContext, command: string): Promise<ExecuteCommandReply>;
  refreshCache(context: BackupClientContext, request: AsyncIterable<RefreshCacheRequest>): Promise<RefreshCacheReply>;
  downloadFileList(context: BackupClientContext, backupShare: Share): AsyncIterableX<FileManifestJournalEntry>;
  copyChunk(context: BackupClientContext, chunk: ChunkInformation): Readable;
  close(context: BackupClientContext): void;
}
