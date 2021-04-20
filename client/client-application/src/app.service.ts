import { BadRequestException, Injectable } from '@nestjs/common';
import {
  FileChunk,
  FileReader,
  GetChunkRequest,
  globStringToRegex,
  LaunchBackupReply,
  LaunchBackupRequest,
  Manifest,
  ManifestService,
  RefreshCacheReply,
  RefreshCacheRequest,
  silence,
  StatusCode,
} from '@woodstock/shared';
import { createReadStream } from 'fs';
import { from, Observable, of, throwError } from 'rxjs';
import { catchError, concatMap, filter, last, map, reduce, switchMap, tap } from 'rxjs/operators';

@Injectable()
export class AppService {
  constructor(private fileReader: FileReader, private manifestService: ManifestService) {}

  launchBackup(request: Observable<LaunchBackupRequest>): Observable<LaunchBackupReply> {
    let manifest: Manifest;

    const journalEntry$ = request.pipe(
      concatMap((value) => {
        if (!!value.header) {
          manifest = new Manifest(`backups.${value.header.sharePath.toString('base64')}`, '/tmp/');
          return from(this.manifestService.exists(manifest)).pipe(
            tap((value) => {
              if (!value) {
                return throwError({ needRefreshCache: true });
              }
            }),
            switchMap(() => this.manifestService.loadIndex(manifest)),
            switchMap((index) =>
              this.fileReader.getFiles(
                index,
                value.header.sharePath,
                value.header.includes?.map((s) => globStringToRegex(s.toString('latin1'))),
                value.header.excludes?.map((s) => globStringToRegex(s.toString('latin1'))),
              ),
            ),
            map((manifestEntry) => this.manifestService.toAddJournalEntry(manifestEntry, true)),
          );
        }
        if (!!value.footer) {
          if (value.footer.code === StatusCode.Ok) {
            return this.manifestService.compact(manifest).pipe(
              last(),
              map(() => ({ response: { code: StatusCode.Ok, needRefreshCache: false } })),
              catchError(() => of({ response: { code: StatusCode.Failed, needRefreshCache: false } })),
            );
          } else {
            return from(this.manifestService.deleteManifest(manifest)).pipe(
              last(),
              map(() => ({ response: { code: StatusCode.Failed, needRefreshCache: false } })),
            );
          }
        }

        return of(value.entry);
      }),
      tap((e) => console.log('e', e, manifest)),
      this.manifestService.writeJournalEntry(() => manifest),

      map((entry) => ({ entry })),
      catchError((err) => {
        console.log(err);
        return of({
          response: { code: StatusCode.Failed, message: err.message, needRefreshCache: !!err.needRefreshCache },
        });
      }),
    );

    return journalEntry$;
  }

  refreshCache(request: Observable<RefreshCacheRequest>): Observable<RefreshCacheReply> {
    let manifest: Manifest;

    const journalEntry$ = request.pipe(
      concatMap((value) => {
        if (!!value.header) {
          manifest = new Manifest(`backups.${value.header.sharePath.toString('base64')}`, '/tmp/');
          return from(this.manifestService.deleteManifest(manifest)).pipe(silence);
        }
        return of(value);
      }),
      filter((request) => !!request.manifest),
      map((request) => request.manifest),
      map((manifest) => this.manifestService.toAddJournalEntry(manifest, true)),
      this.manifestService.writeJournalEntry(() => {
        if (!manifest) {
          throw new BadRequestException('The header must be specified to refresh the cache');
        }
        return manifest;
      }),
      reduce((acc) => acc, { code: StatusCode.Ok }),
      catchError((err) => {
        return of({ code: StatusCode.Failed, message: err.message });
      }),
    );

    // FIXME: should compact
    return journalEntry$;
  }

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
}
