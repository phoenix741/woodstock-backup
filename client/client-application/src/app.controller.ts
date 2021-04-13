import { Controller } from '@nestjs/common';
import { GrpcMethod, GrpcStreamMethod } from '@nestjs/microservices';
import {
  RefreshCacheReply,
  FileManifest,
  LaunchBackupRequest,
} from './app.model';
import { Observable } from 'rxjs';
import { FileChunk, GetChunkRequest, LogEntry } from './app.model';
import {
  FileManifestJournalEntry,
  EntryType,
  UpdateManifestReply,
} from './app.model';

import {
  ExecuteCommandRequest,
  ExecuteCommandReply,
  PrepareBackupRequest,
  PrepareBackupReply,
} from './app.model';

@Controller()
export class AppController {
  @GrpcMethod('WoodstockClientService', 'ExecuteCommand')
  executeCommand(request: ExecuteCommandRequest): ExecuteCommandReply {
    console.log('executeCommand', request.command);
    return {
      code: 0,
      stdout: '',
      stderr: '',
    };
  }

  @GrpcMethod('WoodstockClientService', 'PrepareBackup')
  prepareBackup(request: PrepareBackupRequest): PrepareBackupReply {
    console.log(
      'prepareBackup',
      request.lastBackupNumber,
      request.newBackupNumber,
      request.configuration.sharePath,
    );
    return {
      code: 0,
      needRefreshCache: true,
    };
  }

  @GrpcStreamMethod('WoodstockClientService', 'RefreshCache')
  refreshCache(request: Observable<FileManifest>): RefreshCacheReply {
    console.log('refreshCache', 'start');
    request.subscribe({
      next: (f) => console.log('refreshCache', f),
      complete: () => console.log('refreshCache', 'complete'),
      error: (err) => console.log('refreshCache', 'err', err),
    });

    return {
      code: 0,
    };
  }

  @GrpcMethod('WoodstockClientService', 'LaunchBackup')
  launchBackup(
    request: LaunchBackupRequest,
  ): Observable<FileManifestJournalEntry> {
    console.log(
      'launchBackup',
      request.newBackupNumber,
      request.lastBackupNumber,
      request.configuration.sharePath,
    );

    return new Observable((subscribe) => {
      subscribe.next({
        type: EntryType.ADD,
        manifest: {
          path: Buffer.from('/node_modules'),
        },
      });
      subscribe.next({
        type: EntryType.CLOSE,
      });
      subscribe.complete();
    });
  }

  @GrpcStreamMethod('WoodstockClientService', 'UpdateFileManifest')
  updateFileManifest(
    request: Observable<FileManifestJournalEntry>,
  ): UpdateManifestReply {
    console.log('updateFileManifest', 'start');
    request.subscribe({
      next: (f) => console.log('updateFileManifest', f),
      complete: () => console.log('updateFileManifest', 'complete'),
      error: (err) => console.log('updateFileManifest', 'err', err),
    });

    return {
      code: 0,
    };
  }

  @GrpcMethod('WoodstockClientService', 'GetChunk')
  getChunk(request: GetChunkRequest): Observable<FileChunk> {
    console.log('getChunk', request.filename, request.position, request.size);

    return new Observable((subscribe) => {
      subscribe.next({
        data: Buffer.from('.............'),
      });
      subscribe.complete();
    });
  }

  @GrpcMethod('WoodstockClientService', 'StreamLog')
  streamLog(): Observable<LogEntry> {
    console.log('streamLog');

    return new Observable((subscribe) => {
      subscribe.next({
        level: 0,
        line: 'line',
      });
      subscribe.complete();
    });
  }
}
