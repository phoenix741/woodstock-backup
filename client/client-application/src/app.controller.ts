import { Controller } from '@nestjs/common';
import { GrpcMethod, GrpcStreamMethod } from '@nestjs/microservices';
import {
  ExecuteCommandReply,
  ExecuteCommandRequest,
  FileChunk,
  FileReader,
  GetChunkRequest,
  globStringToRegex,
  LaunchBackupReply,
  LaunchBackupRequest,
  LogEntry,
  Manifest,
  RefreshCacheReply,
  RefreshCacheRequest,
  StatusCode,
} from '@woodstock/shared';
import { createReadStream } from 'fs';
import { concat, forkJoin, from, iif, merge, Observable, of, throwError } from 'rxjs';
import { catchError, filter, first, last, map, reduce, switchMap } from 'rxjs/operators';

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

  @GrpcStreamMethod('WoodstockClientService', 'LaunchBackup')
  launchBackup(request: Observable<LaunchBackupRequest>): Observable<LaunchBackupReply> {
    console.log('launchBackup');

    const shareInformation$ = request.pipe(
      first((value) => !!value.header),
      map((value) => value.header),
    );

    const footerRequest$ = request.pipe(
      last((value) => !!value.footer),
      map((value) => value.footer),
    );

    const defineManifest$ = shareInformation$.pipe(
      map((header) => new Manifest(`backups.${header.sharePath.toString('base64')}`, '/tmp/')),
      switchMap((manifest) => {
        if (!manifest.exists()) {
          return throwError({ needRefreshCache: true });
        }
        return of(manifest);
      }),
    );

    const loadIndex$ = defineManifest$.pipe(switchMap((manifest) => manifest.loadIndex()));

    const journalEntries$ = forkJoin([shareInformation$, loadIndex$]).pipe(
      switchMap(([header, index]) =>
        this.fileReader
          .getFiles(
            index,
            header.sharePath,
            header.includes.map((s) => globStringToRegex(s.toString('latin1'))),
            header.excludes.map((s) => globStringToRegex(s.toString('latin1'))),
          )
          .pipe(map((manifestEntry) => Manifest.toAddJournalEntry(manifestEntry, true))),
      ),
    );

    const updateManifestEntries$ = request.pipe(
      filter((request) => !!request.entry),
      map((request) => request.entry),
    );

    const concatEntries$ = defineManifest$.pipe(
      switchMap((manifest) =>
        merge(journalEntries$, updateManifestEntries$).pipe(
          manifest.writeJournalEntry(),
          map((manifestEntry) => ({ entry: manifestEntry })),
        ),
      ),
      catchError((err) => of({ response: { code: StatusCode.Failed, needRefreshCache: !!err.needRefreshCache } })),
    );

    const compactManifest$ = forkJoin([defineManifest$, footerRequest$]).pipe(
      switchMap(([manifest, footer]) =>
        iif(
          () => footer.code === StatusCode.Ok,
          manifest.compact().pipe(
            last(),
            map(() => ({ response: { code: StatusCode.Ok, needRefreshCache: false } })),
            catchError(() => of({ response: { code: StatusCode.Failed, needRefreshCache: false } })),
          ),
          from(manifest.deleteManifest()).pipe(
            last(),
            map(() => ({ response: { code: StatusCode.Failed, needRefreshCache: false } })),
          ),
        ),
      ),
    );

    return concat(concatEntries$, compactManifest$);
  }

  @GrpcStreamMethod('WoodstockClientService', 'RefreshCache')
  refreshCache(request: Observable<RefreshCacheRequest>): Observable<RefreshCacheReply> {
    console.log('refreshCache', 'start');

    const defineManifest$ = request.pipe(
      first((value) => !!value.header),
      map((value) => new Manifest(`backups.${value.header.sharePath.toString('base64')}`, '/tmp/')),
      switchMap((manifest) => manifest.deleteManifest().then(() => manifest)),
    );

    const journalEntry$ = defineManifest$.pipe(
      switchMap((manifest) => {
        return request.pipe(
          filter((request) => !!request.manifest),
          map((request) => request.manifest),
          map((manifest) => Manifest.toAddJournalEntry(manifest, true)),
          manifest.writeJournalEntry(),
          catchError(() => {
            return of({ code: StatusCode.Failed });
          }),
          reduce((acc) => acc, { code: StatusCode.Ok }),
        );
      }),
    );

    return journalEntry$;
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
