import { Controller } from '@nestjs/common';
import { GrpcMethod, GrpcStreamMethod } from '@nestjs/microservices';
import {
  ExecuteCommandReply,
  ExecuteCommandRequest,
  FileChunk,
  FileManifest,
  FileManifestJournalEntry,
  FileReader,
  GetChunkRequest,
  globStringToRegex,
  LaunchBackupRequest,
  LogEntry,
  longToBigInt,
  Manifest,
  PrepareBackupReply,
  PrepareBackupRequest,
  RefreshCacheReply,
  StatusCode,
  UpdateManifestReply,
} from '@woodstock/shared';
import { createReadStream } from 'fs';
import { defer, from, Observable, of } from 'rxjs';
import { catchError, map, reduce, switchMap } from 'rxjs/operators';

import { LogService } from './log.service';

@Controller()
export class AppController {
  constructor(private fileReader: FileReader, private logService: LogService) {}

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
  prepareBackup(request: PrepareBackupRequest): Observable<PrepareBackupReply> {
    console.log('prepareBackup', request.lastBackupNumber, request.newBackupNumber, request.configuration.sharePath);

    const { sharePath } = request.configuration;

    const manifest = new Manifest(`backups.${sharePath.toString('base64')}`, '/tmp/');

    return from(async () => ({
      code: StatusCode.Ok,
      needRefreshCache: await manifest.exists(),
    }));
  }

  @GrpcStreamMethod('WoodstockClientService', 'RefreshCache')
  refreshCache(request: Observable<FileManifest>): Observable<RefreshCacheReply> {
    console.log('refreshCache', 'start');

    const sharePath = Buffer.from('test'); //FIXME
    const manifest = new Manifest(`backups.${sharePath.toString('base64')}`, '/tmp/');

    const deleteManifest$ = defer(() => manifest.deleteManifest());
    return deleteManifest$.pipe(
      switchMap(() => request),
      map((manifest) => Manifest.toAddJournalEntry(manifest, true)),
      manifest.writeJournalEntry(),
      catchError(() => {
        return of({ code: StatusCode.Failed });
      }),
      reduce((acc) => acc, { code: StatusCode.Ok }),
    );
  }

  @GrpcMethod('WoodstockClientService', 'LaunchBackup')
  launchBackup(request: LaunchBackupRequest): Observable<FileManifestJournalEntry> {
    console.log('launchBackup', request.newBackupNumber, request.lastBackupNumber, request.configuration.sharePath);

    const { sharePath, includes, excludes } = request.configuration;

    const manifest = new Manifest(`backups.${sharePath.toString('base64')}`, '/tmp/');

    const loadIndex$ = manifest.loadIndex();
    const journalEntries$ = loadIndex$
      .pipe(
        switchMap((index) =>
          this.fileReader.getFiles(
            index,
            sharePath,
            includes.map((s) => globStringToRegex(s.toString('latin1'))),
            excludes.map((s) => globStringToRegex(s.toString('latin1'))),
          ),
        ),
      )
      .pipe(
        map((manifest) => Manifest.toAddJournalEntry(manifest, true)),
        manifest.writeJournalEntry(),
      );

    return journalEntries$;
  }

  @GrpcStreamMethod('WoodstockClientService', 'UpdateFileManifest')
  updateFileManifest(request: Observable<FileManifestJournalEntry>): Observable<UpdateManifestReply> {
    console.log('updateFileManifest', 'start');

    const sharePath = Buffer.from('test'); //FIXME
    const manifest = new Manifest(`backups.${sharePath.toString('base64')}`, '/tmp/');
    return request.pipe(
      manifest.writeJournalEntry(),
      catchError(() => {
        return of({ code: StatusCode.Failed });
      }),
      reduce((acc) => acc, { code: StatusCode.Ok }),
    );
  }

  @GrpcMethod('WoodstockClientService', 'GetChunk')
  getChunk(request: GetChunkRequest): Observable<FileChunk> {
    console.log('getChunk', request.filename, request.position, request.size);

    return new Observable<FileChunk>((subscribe) => {
      const { filename, position, size } = request;

      const stream = createReadStream(filename, {
        start: position.toNumber(),
        end: position.add(size).toNumber(),
      });
      stream.on('data', (message: Buffer) => subscribe.next({ data: message }));
      stream.on('end', () => subscribe.complete());
      stream.on('error', (err) => subscribe.error(err));
    });
  }

  @GrpcMethod('WoodstockClientService', 'StreamLog')
  streamLog(): Observable<LogEntry> {
    console.log('streamLog');

    return this.logService.getLogAsObservable();
  }
}
