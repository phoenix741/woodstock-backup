import { BadRequestException, Injectable, InternalServerErrorException, Logger } from '@nestjs/common';
import {
  FileChunk,
  FileHashReader,
  FileReader,
  GetChunkRequest,
  globStringToRegex,
  LaunchBackupHeader,
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
import { concat, defer, EMPTY, from, iif, merge, Observable, of, partition, throwError } from 'rxjs';
import { catchError, concatMap, filter, first, last, map, reduce, shareReplay, switchMap, tap } from 'rxjs/operators';

@Injectable()
export class AppService {
  private logger = new Logger(AppService.name);

  constructor(private fileReader: FileReader, private manifestService: ManifestService) {}

  launchBackup(request: Observable<LaunchBackupRequest>): Observable<LaunchBackupReply> {
    let manifest: Manifest;
    let statusCode: StatusCode;
    let header: LaunchBackupHeader;

    const [headerEntries$, requestEntries$] = partition(request, (value) => !!value.header);
    const [footerEntries$, journalEntries$] = partition(requestEntries$, (value) => !!value.footer);

    const createIndex$ = headerEntries$.pipe(
      first(),
      switchMap((value) => {
        header = value.header;
        manifest = new Manifest(`backups.${header.sharePath.toString('base64')}`, '/tmp/');
        return from(this.manifestService.exists(manifest)).pipe(
          switchMap((manifestExists) => {
            if (!manifestExists && header.lastBackupNumber >= 0) {
              this.logger.warn(`Need refresh cache for the ${header.sharePath.toString()}`);
              return throwError({ needRefreshCache: true });
            }
            return of(value);
          }),
          switchMap(() => this.manifestService.loadIndex(manifest)),
        );
      }),
      shareReplay(1),
    );

    const addRemoveToIndex$ = createIndex$.pipe(
      switchMap((index) =>
        index.walk().pipe(
          filter((entry) => !entry.markViewed),
          map((file) => this.manifestService.toRemoveJournalEntry(file.path)),
          map((entry) => ({ from: 'disk', entry, response: undefined })),
        ),
      ),
    );

    const fromDiskEntries$ = concat(
      createIndex$.pipe(
        switchMap((index) =>
          this.fileReader.getFiles(
            index,
            header.sharePath,
            header.includes?.map((s) => globStringToRegex(s.toString('latin1'))),
            header.excludes?.map((s) => globStringToRegex(s.toString('latin1'))),
          ),
        ),
        map((entry) => this.manifestService.toAddJournalEntry(entry, true)),
        map((entry) => ({ from: 'disk', entry, response: undefined })),
      ),
      addRemoveToIndex$,
      of({ from: 'disk', entry: undefined, response: { code: StatusCode.Partial, diskReadFinished: true } }),
    );

    const fromRequestEntries$ = journalEntries$.pipe(
      tap((e) => this.logger.log(`From request: ${JSON.stringify(e)}`)),
      map((manifestEntry) => ({ from: 'network', entry: manifestEntry.entry, response: undefined })),
    );

    const entries$ = merge(fromDiskEntries$, fromRequestEntries$).pipe(
      this.manifestService.writeJournalEntry(
        () => manifest,
        (entry) => entry.entry,
      ),
      filter((entry) => entry.from === 'disk'),
      map(({ entry, response }) => ({ entry, response })),
    );

    const statusCode$ = footerEntries$.pipe(
      first(),
      switchMap((value) => {
        statusCode = value.footer.code;
        return EMPTY;
      }),
    );

    const compactEntries$ = iif(
      () => statusCode === StatusCode.Ok,
      defer(() =>
        this.manifestService.compact(manifest).pipe(reduce((acc) => acc, { response: { code: StatusCode.Ok } })),
      ),
      defer(() =>
        from(this.manifestService.deleteManifest(manifest)).pipe(
          last(),
          map(() => ({ response: { code: StatusCode.Failed } })),
        ),
      ),
    );

    const journalEntry$ = concat(merge(entries$, statusCode$), compactEntries$).pipe(
      catchError((err) => {
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
    return new Observable<FileChunk>((subscribe) => {
      const { filename, position, size, sha256 } = request;

      const stream = createReadStream(filename, {
        start: position.toNumber(),
        end: position.add(size).sub(1).toNumber(),
      });
      const hashReader = new FileHashReader();

      hashReader.on('data', (message: Buffer) => subscribe.next({ data: message }));
      hashReader.on('end', () => {
        if (hashReader.hash && !hashReader.hash.equals(sha256)) {
          const message = `The chunk ${filename.toString()}:${position}:${size} should have a sha of ${sha256.toString(
            'hex',
          )}, but is ${hashReader.hash.toString('hex')}`;
          this.logger.warn(message);
          return subscribe.error(new InternalServerErrorException(message));
        }
        subscribe.complete();
      });
      hashReader.on('error', (err) => subscribe.error(err));

      stream.pipe(hashReader);
    });
  }
}
