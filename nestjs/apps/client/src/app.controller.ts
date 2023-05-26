import { Metadata, ServerReadableStream } from '@grpc/grpc-js';
import { BadRequestException, Controller, InternalServerErrorException, Logger, UseGuards } from '@nestjs/common';
import { GrpcMethod, GrpcStreamCall, GrpcStreamMethod } from '@nestjs/microservices';
import {
  AuthenticateReply,
  AuthenticateRequest,
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
import { catchError, map, startWith } from 'ix/asynciterable/operators';
import { concatMap, from, Observable, of } from 'rxjs';
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
    this.logger.log('WoodstockClientService.Authenticate');

    return await this.service.authenticate(request);
  }

  @UseGuards(AuthGuard)
  @GrpcMethod('WoodstockClientService', 'ExecuteCommand')
  async executeCommand(request: ExecuteCommandRequest): Promise<ExecuteCommandReply> {
    this.logger.log(`WoodstockClientService.ExecuteCommand: ${request.command}`);

    try {
      return await this.service.executeCommand(request.command);
    } catch (err) {
      this.logger.error(`Can't execute the command ${request.command}: ${err.message}`);
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
    this.logger.log('WoodstockClientService.RefreshCache');

    try {
      const reply = await this.service.refreshCache(fromIx(requestStream));
      callback(null, reply);
    } catch (err) {
      this.logger.error(`Can't execute refresh cache: ${err.message}`);
      callback(null, {
        code: StatusCode.Failed,
        message: err.message,
      });
    }
  }

  @UseGuards(AuthGuard)
  @GrpcMethod('WoodstockClientService', 'LaunchBackup')
  launchBackup(request: LaunchBackupRequest): Observable<LaunchBackupReply> {
    this.logger.log(`WoodstockClientService.LaunchBackup: ${request.share?.sharePath?.toString()}`);

    try {
      if (!request.share) {
        throw new InternalServerErrorException('share must be defined');
      }

      return from(this.service.launchBackup(request.share));
    } catch (err) {
      this.logger.error(`Can't launch the backup: ${err.message}`);
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
  @GrpcStreamMethod('WoodstockClientService', 'GetChunk')
  getChunk(request: Observable<GetChunkRequest>): Observable<GetChunkReply> {
    this.logger.log('WoodstockClientService.GetChunk');

    try {
      return request.pipe(
        concatMap((request) => {
          if (!request.chunk) {
            throw new InternalServerErrorException('chunk must be defined');
          }
          return fromIx(this.service.getChunk(request.chunk)).pipe(
            map((data) => ({ data })),
            startWith({ chunk: request.chunk }),
            catchError((err) => {
              this.logger.error(err);
              return ofIx({ error: { code: StatusCode.Failed } });
            }),
          );
        }),
      );
    } catch (err) {
      this.logger.error(`Can't get all chunks: ${err.message}`);
      return of({ error: { code: StatusCode.Failed } });
    }
  }

  @UseGuards(AuthGuard)
  @GrpcMethod('WoodstockClientService', 'StreamLog')
  streamLog(_: StreamLogRequest): Observable<LogEntry> {
    this.logger.log('WoodstockClientService.StreamLog');

    return this.service.getLogAsObservable();
  }

  @UseGuards(AuthGuard)
  @GrpcMethod('WoodstockClientService', 'CloseClient')
  async closeBackup(request: Empty, metadata: Metadata): Promise<Empty> {
    this.logger.log('WoodstockClientService.CloseClient');

    const sessionId = getMetadata<string>(metadata, 'X-Session-Id');
    await this.service.closeBackup(sessionId);

    return {};
  }
}
