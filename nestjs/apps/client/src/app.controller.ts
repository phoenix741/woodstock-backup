import { Metadata, ServerReadableStream } from '@grpc/grpc-js';
import { BadRequestException, Controller, InternalServerErrorException, Logger, UseGuards } from '@nestjs/common';
import { GrpcMethod, GrpcStreamCall } from '@nestjs/microservices';
import {
  AuthenticateReply,
  AuthenticateRequest,
  ChunkStatus,
  Empty,
  ExecuteCommandReply,
  ExecuteCommandRequest,
  GetChunkReply,
  GetChunkRequest,
  LaunchBackupReply,
  LaunchBackupRequest,
  LogEntry,
  RefreshCacheReply,
  RefreshCacheRequest,
  StatusCode,
  StreamLogRequest,
} from '@woodstock/shared';
import { from as fromIx, of as ofIx } from 'ix/asynciterable';
import { catchError, map } from 'ix/asynciterable/operators';
import { from, Observable, of } from 'rxjs';
import { AppService } from './app.service.js';
import { AuthGuard } from './auth/auth.guard.js';

function getMetadata<T extends string | Buffer>(metadata: Metadata, key: string): T {
  const value = metadata.get(key);
  if (!value || !value.length) {
    throw new BadRequestException(`Missing metadata key: ${key}`);
  }
  return value[0] as T;
}

@Controller()
export class AppController {
  private logger = new Logger(AppController.name);

  constructor(private service: AppService) {}

  @GrpcMethod('WoodstockClientService', 'Authenticate')
  async authenticate(request: AuthenticateRequest): Promise<AuthenticateReply> {
    return await this.service.authenticate(request);
  }

  @UseGuards(AuthGuard)
  @GrpcMethod('WoodstockClientService', 'ExecuteCommand')
  async executeCommand(request: ExecuteCommandRequest): Promise<ExecuteCommandReply> {
    try {
      return await this.service.executeCommand(request.command);
    } catch (err) {
      return {
        code: StatusCode.Failed,
        stdout: '',
        stderr: err.message,
      };
    }
  }

  @UseGuards(AuthGuard)
  @GrpcStreamCall('WoodstockClientService', 'RefreshCache')
  async refreshCache(
    requestStream: ServerReadableStream<RefreshCacheRequest, RefreshCacheReply>,
    callback: (err: unknown, value: RefreshCacheReply) => void,
  ) {
    try {
      const reply = await this.service.refreshCache(fromIx(requestStream));
      callback(null, reply);
    } catch (err) {
      callback(null, {
        code: StatusCode.Failed,
        message: err.message,
      });
    }
  }

  @UseGuards(AuthGuard)
  @GrpcMethod('WoodstockClientService', 'LaunchBackup')
  launchBackup(request: LaunchBackupRequest): Observable<LaunchBackupReply> {
    try {
      if (!request.share) {
        throw new InternalServerErrorException('share must be defined');
      }

      return from(this.service.launchBackup(request.share));
    } catch (err) {
      return of({
        entry: undefined,
        response: {
          code: StatusCode.Failed,
          message: err.message,
        },
      });
    }
  }

  @UseGuards(AuthGuard)
  @GrpcMethod('WoodstockClientService', 'GetChunk')
  getChunk(request: GetChunkRequest): Observable<GetChunkReply> {
    try {
      if (!request.chunk) {
        throw new InternalServerErrorException('chunk must be defined');
      }

      return from(
        this.service.getChunk(request.chunk).pipe(
          map((data) => ({ data, status: ChunkStatus.DATA })),
          catchError((err) => {
            this.logger.error(err);
            return ofIx({ data: undefined, status: ChunkStatus.ERROR });
          }),
        ),
      );
    } catch (err) {
      this.logger.error(err);
      return of({ data: undefined, status: ChunkStatus.ERROR });
    }
  }

  @UseGuards(AuthGuard)
  @GrpcMethod('WoodstockClientService', 'StreamLog')
  streamLog(_: StreamLogRequest): Observable<LogEntry> {
    return this.service.getLogAsObservable();
  }

  @UseGuards(AuthGuard)
  @GrpcMethod('WoodstockClientService', 'CloseClient')
  async closeBackup(request: Empty, metadata: Metadata): Promise<Empty> {
    const sessionId = getMetadata<string>(metadata, 'X-Session-Id');
    await this.service.closeBackup(sessionId);

    return {};
  }
}
