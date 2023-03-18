import { Metadata, ServerReadableStream } from '@grpc/grpc-js';
import { BadRequestException, Controller, InternalServerErrorException, Logger } from '@nestjs/common';
import { GrpcMethod, GrpcStreamCall } from '@nestjs/microservices';
import {
  AuthenticateReply,
  AuthenticateRequest,
  ChunkStatus,
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

  @GrpcMethod('WoodstockClientService', 'ExecuteCommand')
  async executeCommand(request: ExecuteCommandRequest, metadata: Metadata): Promise<ExecuteCommandReply> {
    try {
      const sessionId = getMetadata<string>(metadata, 'X-Session-Id');

      return await this.service.executeCommand(sessionId, request.command);
    } catch (err) {
      return {
        code: StatusCode.Failed,
        stdout: '',
        stderr: err.message,
      };
    }
  }

  @GrpcStreamCall('WoodstockClientService', 'RefreshCache')
  async refreshCache(
    requestStream: ServerReadableStream<RefreshCacheRequest, RefreshCacheReply>,
    callback: (err: unknown, value: RefreshCacheReply) => void,
  ) {
    const sessionId = getMetadata<string>(requestStream.metadata, 'X-Session-Id');
    try {
      const reply = await this.service.refreshCache(sessionId, fromIx(requestStream));
      callback(null, reply);
    } catch (err) {
      callback(null, {
        code: StatusCode.Failed,
        message: err.message,
      });
    }
  }

  @GrpcMethod('WoodstockClientService', 'LaunchBackup')
  launchBackup(request: LaunchBackupRequest, metadata: Metadata): Observable<LaunchBackupReply> {
    try {
      if (!request.share) {
        throw new InternalServerErrorException('share must be defined');
      }
      const sessionId = getMetadata<string>(metadata, 'X-Session-Id');
      return from(this.service.launchBackup(sessionId, request.share));
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

  @GrpcMethod('WoodstockClientService', 'GetChunk')
  getChunk(request: GetChunkRequest, metadata: Metadata): Observable<GetChunkReply> {
    try {
      if (!request.chunk) {
        throw new InternalServerErrorException('chunk must be defined');
      }

      const sessionId = getMetadata<string>(metadata, 'X-Session-Id');
      return from(
        this.service.getChunk(sessionId, request.chunk).pipe(
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

  @GrpcMethod('WoodstockClientService', 'StreamLog')
  streamLog(_: StreamLogRequest, metadata: Metadata): Observable<LogEntry> {
    const sessionId = getMetadata<string>(metadata, 'X-Session-Id');
    return this.service.getLogAsObservable(sessionId);
  }
}
