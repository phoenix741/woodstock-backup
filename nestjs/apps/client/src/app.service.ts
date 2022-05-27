import { Injectable, InternalServerErrorException } from '@nestjs/common';
import {
  AuthenticateReply,
  AuthenticateRequest,
  ChunkInformation,
  EncryptionService,
  FileChunk,
  LaunchBackupReply,
  LogEntry,
  RefreshCacheReply,
  RefreshCacheRequest,
  Share,
  StatusCode,
} from '@woodstock/shared';
import { AsyncIterableX } from 'ix/asynciterable';
import { Observable } from 'rxjs';
import { BackupService } from './backup/backup.service';
import { ClientConfigService } from './config/client.config';
import { LogService } from './logger/log.service';

@Injectable()
export class AppService {
  constructor(
    private clientConfig: ClientConfigService,
    private encryptionService: EncryptionService,
    private backupService: BackupService,
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

      const uuid = this.backupService.initializeBackup(request);
      return { code: StatusCode.Ok, sessionId: uuid };
    } catch (err) {
      return {
        code: StatusCode.Failed,
        message: err.message,
      };
    }
  }

  async refreshCache(sessionId: string, request: AsyncIterableX<RefreshCacheRequest>): Promise<RefreshCacheReply> {
    const context = this.backupService.getContext(sessionId);
    if (!context) {
      throw new InternalServerErrorException('The context is not defined');
    }

    return context.refreshCache(request);
  }

  launchBackup(sessionId: string, share: Share): AsyncIterableX<LaunchBackupReply> {
    const context = this.backupService.getContext(sessionId);
    if (!context) {
      throw new InternalServerErrorException('The context is not defined');
    }

    return context.launchBackup(share);
  }

  getChunk(sessionId: string, request: ChunkInformation): AsyncIterableX<FileChunk> {
    const context = this.backupService.getContext(sessionId);
    if (!context) {
      throw new InternalServerErrorException('The context is not defined');
    }

    return context.getChunk(request);
  }

  getLogAsObservable(sessionId: string): Observable<LogEntry> {
    this.backupService.getContext(sessionId);
    return this.logService.getLogAsObservable();
  }
}
