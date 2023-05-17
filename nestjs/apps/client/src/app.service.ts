import { BadRequestException, Injectable } from '@nestjs/common';
import {
  AuthenticateReply,
  AuthenticateRequest,
  BackupOnClientService,
  ChunkInformation,
  ExecuteCommandReply,
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
import { AuthService } from './auth/auth.service.js';
import { LogService } from './log.service.js';

export interface JwtPayload {
  sessionId: string;
}

@Injectable()
export class AppService {
  constructor(
    private authService: AuthService,
    private backupService: BackupOnClientService,
    private logService: LogService,
  ) {}

  async authenticate(request: AuthenticateRequest): Promise<AuthenticateReply> {
    try {
      if (request.version !== 0) {
        throw new BadRequestException('Unsupported version');
      }

      const sessionToken = await this.authService.authenticate(request.token);

      return { code: StatusCode.Ok, sessionId: sessionToken };
    } catch (err) {
      return {
        code: StatusCode.Failed,
        message: err.message,
      };
    }
  }

  async executeCommand(command: string): Promise<ExecuteCommandReply> {
    return this.backupService.executeCommand(command);
  }

  async refreshCache(request: AsyncIterableX<RefreshCacheRequest>): Promise<RefreshCacheReply> {
    return this.backupService.refreshCache(request);
  }

  launchBackup(share: Share): AsyncIterableX<LaunchBackupReply> {
    return this.backupService.launchBackup(share);
  }

  getChunk(request: ChunkInformation): AsyncIterableX<FileChunk> {
    return this.backupService.getChunk(request);
  }

  getLogAsObservable(): Observable<LogEntry> {
    return this.logService.getLogAsObservable();
  }

  async closeBackup(sessionToken: string): Promise<void> {
    this.authService.logout(sessionToken);
  }
}
