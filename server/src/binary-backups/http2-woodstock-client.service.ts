import { HttpService } from '@nestjs/common';
import { Observable, EMPTY } from 'rxjs';
import { map, switchMap, tap } from 'rxjs/operators';
import { FileManifestJournalEntry } from 'src/storage/backup-manifest/manifest.model';
import { Readable } from 'stream';
import * as http2 from 'http2-wrapper';

import {
  ProtobufMessageReader,
  ProtobufMessageWithPosition,
  ProtoFileManifestJournalEntry,
  ProtoGetChunkRequest,
  ProtoLaunchBackupRequest,
  ProtoPrepareBackupReply,
  ProtoPrepareBackupRequest,
} from '../storage/backup-manifest/proto.utils';
import {
  FileChunk,
  GetChunkRequest,
  LaunchBackupRequest,
  PrepareBackupReply,
  PrepareBackupRequest,
  WoodstockClientService,
} from './models/woodstock-client.service';
// import { readFileSync } from 'fs';
import { AxiosRequestConfig } from 'axios';
import { StatusCode } from './models/woodstock-client.service';

process.env.NODE_TLS_REJECT_UNAUTHORIZED = '0';

export class Http2WoodstockClientService implements WoodstockClientService {
  constructor(private httpService: HttpService, private hostToBackup: string) {}

  prepareBackup(request: PrepareBackupRequest): Observable<PrepareBackupReply> {
    const data = ProtoPrepareBackupRequest.encode(request).finish();
    return this.httpService
      .request<Buffer>({
        method: 'POST',
        url: `https://${this.hostToBackup}:3000/prepare-backup`,
        data,
        transport: http2,
        responseType: 'arraybuffer',
      } as AxiosRequestConfig)
      .pipe(
        // tap((r) => console.log(r.data)),
        map((response) =>
          response.data.length
            ? (ProtoPrepareBackupReply.toObject(ProtoPrepareBackupReply.decode(response.data)) as PrepareBackupReply)
            : { code: StatusCode.Ok, needRefreshCache: false },
        ),
      );
  }

  launchBackup(request: LaunchBackupRequest): Observable<FileManifestJournalEntry> {
    const data = ProtoLaunchBackupRequest.encode(request).finish();
    return this.httpService
      .request<Readable>({
        method: 'post',
        url: `https://${this.hostToBackup}:3000/launch-backup`,
        data,
        transport: http2,
        responseType: 'stream',
      } as AxiosRequestConfig)
      .pipe(
        switchMap(
          (response) =>
            new Observable<FileManifestJournalEntry>((subscribe) => {
              const transform = new ProtobufMessageReader(ProtoFileManifestJournalEntry, undefined, false);

              transform.on('data', (message: ProtobufMessageWithPosition<FileManifestJournalEntry>) =>
                subscribe.next(message.message),
              );
              transform.on('end', () => subscribe.complete());
              transform.on('error', (err) => subscribe.error(err));
              response.data.on('error', (err) => subscribe.error(err));

              response.data.pipe(transform);
            }),
        ),
      );
  }

  getChunk(request: GetChunkRequest): Observable<FileChunk> {
    const data = ProtoGetChunkRequest.encode(request).finish();
    return this.httpService
      .request<Readable>({
        method: 'post',
        url: `https://${this.hostToBackup}:3000/get-chunk`,
        data,
        transport: http2,
        responseType: 'stream',
      } as AxiosRequestConfig)
      .pipe(
        switchMap(
          (response) =>
            new Observable<FileChunk>((subscribe) => {
              response.data.on('data', (data: Buffer) => subscribe.next({ data }));
              response.data.on('error', (err) => subscribe.error(err));
              response.data.on('end', () => subscribe.complete());
            }),
        ),
      );
  }
  /*
  getChunk(request: GetChunkRequest): Observable<Readable> {
    const data = ProtoGetChunkRequest.encode(request).finish();
    return this.httpService
      .request<Readable>({
        method: 'post',
        url: `https://${this.hostToBackup}:3000/get-chunk`,
        data,
        transport: http2,
        responseType: 'stream',
      } as AxiosRequestConfig)
      .pipe(map((response) => response.data));
  }
  */
}
