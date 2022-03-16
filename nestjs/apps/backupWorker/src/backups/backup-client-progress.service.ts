import { Injectable, LoggerService } from '@nestjs/common';
import { isPoolChunkInformation, TaskProgression } from '@woodstock/backoffice-shared';
import { EntryType, longToBigInt, Share } from '@woodstock/shared';
import * as Long from 'long';
import { defer, endWith, map, mapTo, Observable, scan, startWith } from 'rxjs';
import { BackupsGrpcContext } from './backup-client-grpc.class';
import { BackupClient } from './backup-client.service';

@Injectable()
export class BackupClientProgress {
  constructor(private backupClient: BackupClient) {}

  authenticate(context: BackupsGrpcContext, logger: LoggerService): Observable<TaskProgression> {
    const authenticate$ = defer(() => this.backupClient.authenticate(context, logger));

    return authenticate$.pipe(
      mapTo(new TaskProgression()),
      endWith(new TaskProgression({ percent: 100 })),
      startWith(new TaskProgression({ percent: 0 })),
    );
  }

  executeCommand(context: BackupsGrpcContext, command: string): Observable<TaskProgression> {
    const executeCommand$ = defer(() => this.backupClient.executeCommand(context, command));

    return executeCommand$.pipe(
      startWith(new TaskProgression({ percent: 0 })),
      mapTo(new TaskProgression()),
      endWith(new TaskProgression({ percent: 100 })),
    );
  }

  getFileList(context: BackupsGrpcContext, backupShare: Share): Observable<TaskProgression> {
    const fileList$ = this.backupClient.getFileList(context, backupShare).pipe(
      scan((current, value) => {
        return new TaskProgression({
          progressMax: current.progressMax + longToBigInt(value.manifest?.stats?.size || Long.ZERO),
        });
      }, new TaskProgression({ percent: 0, progressMax: 0n })),
    );

    return fileList$;
  }

  createBackup(context: BackupsGrpcContext, backupShare: Share): Observable<TaskProgression> {
    return this.backupClient.createBackup(context, backupShare).pipe(
      scan(
        (current, value) => {
          if (isPoolChunkInformation(value)) {
            return {
              ...current,
              progress: current.progress + value.size,
            };
          } else {
            switch (value.type) {
              case EntryType.ADD:
                return {
                  ...current,
                  count: current.count + 1,
                  size: current.size + longToBigInt(value.manifest?.stats?.size || Long.ZERO),
                  compressedFileSize:
                    current.compressedFileSize + longToBigInt(value.manifest?.stats?.compressedSize || Long.ZERO),
                };
              case EntryType.MODIFY:
                return {
                  ...current,
                  size: current.size + longToBigInt(value.manifest?.stats?.size || Long.ZERO),
                  compressedFileSize:
                    current.compressedFileSize + longToBigInt(value.manifest?.stats?.compressedSize || Long.ZERO),
                };
              case EntryType.REMOVE:
                return current;
            }
          }
        },
        {
          progress: 0n,
          count: 0,
          size: 0n,
          compressedFileSize: 0n,
          date: new Date(),
        },
      ),
      map((fileCount) => {
        const elapsedTime = BigInt(Date.now() - fileCount.date.getTime());
        return new TaskProgression({
          newCompressedFileSize: fileCount.compressedFileSize,
          newFileCount: fileCount.count,
          newFileSize: fileCount.size,
          progressCurrent: fileCount.progress,
          speed: Number(elapsedTime && (fileCount.progress * 1000n) / elapsedTime),
        });
      }),
      startWith(new TaskProgression({ percent: 0 })),
    );
  }

  compact(context: BackupsGrpcContext, sharePath: Buffer): Observable<TaskProgression> {
    return this.backupClient.compact(context, sharePath).pipe(
      scan(
        (current, value) => {
          return {
            ...current,
            progress: current.progress + 1,
            count: current.count + 1,
            size: current.size + longToBigInt(value.stats?.size || Long.ZERO),
            compressedFileSize: current.compressedFileSize + longToBigInt(value.stats?.compressedSize || Long.ZERO),
          };
        },
        {
          progress: 0,
          count: 0,
          size: 0n,
          compressedFileSize: 0n,
        },
      ),
      map(
        (fileCount) =>
          new TaskProgression({
            compressedFileSize: fileCount.compressedFileSize,
            fileCount: fileCount.count,
            fileSize: fileCount.size,
            percent: 0,
          }),
      ),
      startWith(new TaskProgression({ percent: 0 })),
    );
  }

  countRef(context: BackupsGrpcContext): Observable<TaskProgression> {
    const countRef$ = defer(() => this.backupClient.countRef(context));

    return countRef$.pipe(
      startWith(new TaskProgression({ percent: 0 })),
      mapTo(new TaskProgression()),
      endWith(new TaskProgression({ percent: 100 })),
    );
  }

  refreshCache(context: BackupsGrpcContext, shares: string[]): Observable<TaskProgression> {
    const refreshCache$ = defer(() => this.backupClient.refreshCache(context, shares));
    return refreshCache$.pipe(
      mapTo(new TaskProgression()),
      startWith(new TaskProgression({ percent: 0 })),
      endWith(new TaskProgression({ percent: 100 })),
    );
  }

  close(context: BackupsGrpcContext): void {
    this.backupClient.close(context);
  }
}
