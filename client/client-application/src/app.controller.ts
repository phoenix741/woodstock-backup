import { Controller } from '@nestjs/common';
import { GrpcMethod, GrpcStreamMethod } from '@nestjs/microservices';
import {
  EntryType,
  ExecuteCommandReply,
  ExecuteCommandRequest,
  FileChunk,
  GetChunkRequest,
  LaunchBackupReply,
  LaunchBackupRequest,
  LogEntry,
  RefreshCacheReply,
  RefreshCacheRequest,
  StatusCode,
} from '@woodstock/shared';
import { Observable } from 'rxjs';

import { AppService } from './app.service';
import { LogService } from './log.service';
import { tap } from 'rxjs/operators';

@Controller()
export class AppController {
  constructor(private service: AppService, private logService: LogService) {}

  @GrpcMethod('WoodstockClientService', 'ExecuteCommand')
  executeCommand(request: ExecuteCommandRequest): ExecuteCommandReply {
    return {
      code: StatusCode.Ok,
      stdout: request.command,
      stderr: '',
    };
  }

  @GrpcStreamMethod('WoodstockClientService', 'LaunchBackup')
  launchBackup(request: Observable<LaunchBackupRequest>): Observable<LaunchBackupReply> {
    return this.service.launchBackup(request);
  }

  @GrpcStreamMethod('WoodstockClientService', 'RefreshCache')
  refreshCache(request: Observable<RefreshCacheRequest>): Observable<RefreshCacheReply> {
    return this.service.refreshCache(request);
  }

  @GrpcMethod('WoodstockClientService', 'GetChunk')
  getChunk(request: GetChunkRequest): Observable<FileChunk> {
    return this.service.getChunk(request);
  }

  @GrpcMethod('WoodstockClientService', 'StreamLog')
  streamLog(): Observable<LogEntry> {
    console.log('streamLog');

    return this.logService.getLogAsObservable();
  }
}
