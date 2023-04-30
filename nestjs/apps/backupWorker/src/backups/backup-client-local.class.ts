import { Injectable, Logger, LoggerService } from '@nestjs/common';
import {
  AuthenticateReply,
  BackupClientContext,
  BackupLogger,
  ChunkInformation,
  ExecuteCommandReply,
  FileManifestJournalEntry,
  joinBuffer,
  LogEntry,
  RefreshCacheReply,
  RefreshCacheRequest,
  Share,
  StatusCode,
} from '@woodstock/shared';
import { BackupOnClientService } from '@woodstock/shared';
import { AsyncIterableX, from } from 'ix/asynciterable';
import { filter, map } from 'ix/asynciterable/operators';
import { Readable } from 'stream';
import { BackupClientInterface } from './backup-client.interface.js';
import { LaunchBackupError } from './backup.error.js';

export class BackupsLocalContext implements BackupClientContext {
  isLocal = true;
  sessionId?: string;
  ip?: string;
  logger?: BackupLogger;
  abortable: AbortController[] = [];

  constructor(
    public host: string,
    public currentBackupId: number,
    public pathPrefix: string,
    public originalDate?: number,
  ) {}
}
@Injectable()
export class BackupClientLocal implements BackupClientInterface {
  private readonly logger = new Logger(BackupClientLocal.name);

  constructor(private backupService: BackupOnClientService) {}

  createContext(
    hostname: string,
    currentBackupId: number,
    pathPrefix: string,
    originalDate?: number,
  ): BackupsLocalContext {
    this.logger.log(`Create LOCAL context to ${hostname}`);

    return new BackupsLocalContext(hostname, currentBackupId, pathPrefix, originalDate);
  }

  async createConnection(): Promise<void> {
    // nothing to do
  }

  async authenticate(context: BackupClientContext): Promise<AuthenticateReply> {
    context.sessionId = 'local';
    return {
      code: StatusCode.Ok,
      sessionId: 'local',
    };
  }

  streamLog(): AsyncIterableX<LogEntry> {
    return from([]);
  }

  async executeCommand(context: BackupsLocalContext, command: string): Promise<ExecuteCommandReply> {
    const reply = await this.backupService.executeCommand(command);

    reply?.stderr && context.logger?.error(reply.stderr);
    reply?.stdout && context.logger?.log(reply.stdout);

    if (!reply || reply.code) {
      context.logger?.log(`The command "${command}" has been executed with error: ${reply?.code}`, 'executeCommand');
      throw new LaunchBackupError(reply.stderr || `Can\' execute the command ${command}`);
    } else {
      context.logger?.log(`The command "${command}" has been executed successfully.`, 'executeCommand');
    }

    return reply;
  }

  async refreshCache(
    context: BackupsLocalContext,
    request: AsyncIterable<RefreshCacheRequest>,
  ): Promise<RefreshCacheReply> {
    return await this.backupService.refreshCache(
      from(request).pipe(
        map((r) => {
          if (r.header) {
            r.header.sharePath = joinBuffer(Buffer.from(context.pathPrefix), r.header.sharePath);
          }
          return r;
        }),
      ),
    );
  }

  downloadFileList(context: BackupsLocalContext, backupShare: Share): AsyncIterableX<FileManifestJournalEntry> {
    backupShare = { ...backupShare, sharePath: joinBuffer(Buffer.from(context.pathPrefix), backupShare.sharePath) };
    return this.backupService.launchBackup(backupShare).pipe(
      map(({ entry }) => entry),
      filter((entry): entry is FileManifestJournalEntry => !!entry),
    );
  }

  copyChunk(_context: BackupsLocalContext, chunk: ChunkInformation): Readable {
    const chunkResult = this.backupService.getChunk(chunk);
    return Readable.from(
      chunkResult.pipe(
        map((pieceOfChunk) => pieceOfChunk.data),
        filter((buffer): buffer is Buffer => !!buffer),
      ),
    );
  }

  close(): void {
    // Nothing to do
  }
}
