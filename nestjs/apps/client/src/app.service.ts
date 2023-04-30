import { BadRequestException, Injectable, UnauthorizedException } from '@nestjs/common';
import {
  AuthenticateReply,
  AuthenticateRequest,
  ChunkInformation,
  EncryptionService,
  ExecuteCommandReply,
  FileChunk,
  LaunchBackupReply,
  LogEntry,
  RefreshCacheReply,
  RefreshCacheRequest,
  Share,
  StatusCode,
} from '@woodstock/shared';
import { BackupOnClientService } from '@woodstock/shared';
import { AsyncIterableX } from 'ix/asynciterable';
import { Observable } from 'rxjs';
import { v4 as uuidv4 } from 'uuid';
import { ClientConfigService } from './config/client.config.js';
import { LogService } from './logger/log.service.js';

@Injectable()
export class AppService {
  private context = new Set<string>();

  constructor(
    private clientConfig: ClientConfigService,
    private encryptionService: EncryptionService,
    private backupService: BackupOnClientService,
    private logService: LogService,
  ) {}

  async authenticate(request: AuthenticateRequest): Promise<AuthenticateReply> {
    try {
      // Check token validity
      await this.encryptionService.verifyAuthentificationToken(
        this.clientConfig.config.hostname,
        request.token,
        this.clientConfig.config.password,
      );

      if (request.version !== 0) {
        throw new BadRequestException('Unsupported version');
      }

      const uuid = uuidv4();
      this.context.add(uuid);

      return { code: StatusCode.Ok, sessionId: uuid };
    } catch (err) {
      return {
        code: StatusCode.Failed,
        message: err.message,
      };
    }
  }

  checkContext(sessionId: string) {
    if (!this.context.has(sessionId)) {
      throw new UnauthorizedException('Session not found');
    }
  }

  async executeCommand(sessionId: string, command: string): Promise<ExecuteCommandReply> {
    this.checkContext(sessionId);

    return this.backupService.executeCommand(command);
  }

  async refreshCache(sessionId: string, request: AsyncIterableX<RefreshCacheRequest>): Promise<RefreshCacheReply> {
    this.checkContext(sessionId);

    return this.backupService.refreshCache(request);
  }

  launchBackup(sessionId: string, share: Share): AsyncIterableX<LaunchBackupReply> {
    this.checkContext(sessionId);

    return this.backupService.launchBackup(share);
  }

  getChunk(sessionId: string, request: ChunkInformation): AsyncIterableX<FileChunk> {
    this.checkContext(sessionId);

    return this.backupService.getChunk(request);
  }

  getLogAsObservable(sessionId: string): Observable<LogEntry> {
    this.checkContext(sessionId);

    return this.logService.getLogAsObservable();
  }
}
