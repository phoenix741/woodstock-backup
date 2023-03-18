import { Injectable, LoggerService } from '@nestjs/common';
import { BackupClientContext, EntryType, isPoolChunkInformation, longToBigInt, Share } from '@woodstock/shared';
import { QueueTaskProgression } from '@woodstock/shared/tasks';
import * as Long from 'long';
import { defer, endWith, map, mapTo, Observable, scan, startWith } from 'rxjs';
import { BackupClient } from './backup-client.service.js';

@Injectable()
export class BackupClientProgress {
  constructor(private backupClient: BackupClient) {}

  createContext(
    ip: string | undefined,
    hostname: string,
    currentBackupId: number,
    pathPrefix?: string,
  ): BackupClientContext {
    return this.backupClient.createContext(ip, hostname, currentBackupId, pathPrefix);
  }

  async createConnection(context: BackupClientContext): Promise<void> {
    await this.backupClient.createConnection(context);
  }

  authenticate(
    context: BackupClientContext,
    logger: LoggerService,
    clientLogger: LoggerService,
    password: string,
  ): Promise<void> {
    return this.backupClient.authenticate(context, logger, clientLogger, password);
  }

  executeCommand(context: BackupClientContext, command: string): Observable<QueueTaskProgression> {
    const executeCommand$ = defer(() => this.backupClient.executeCommand(context, command));

    return executeCommand$.pipe(
      startWith(new QueueTaskProgression({ progressCurrent: 0n, progressMax: 0n })),
      map(() => new QueueTaskProgression()),
      endWith(new QueueTaskProgression({ progressCurrent: 1n, progressMax: 1n })),
    );
  }

  getFileList(context: BackupClientContext, backupShare: Share): Observable<QueueTaskProgression> {
    const fileList$ = this.backupClient.getFileList(context, backupShare).pipe(
      scan((current, value) => {
        return new QueueTaskProgression({
          progressMax: current.progressMax + longToBigInt(value.manifest?.stats?.size || Long.ZERO),
        });
      }, new QueueTaskProgression({ progressCurrent: 0n, progressMax: 0n })),
    );

    return fileList$;
  }

  createBackup(context: BackupClientContext, backupShare: Share): Observable<QueueTaskProgression> {
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
        return new QueueTaskProgression({
          newCompressedFileSize: fileCount.compressedFileSize,
          newFileCount: fileCount.count,
          newFileSize: fileCount.size,
          progressCurrent: fileCount.progress,
          speed: Number(elapsedTime && (fileCount.progress * 1000n) / elapsedTime),
        });
      }),
      startWith(new QueueTaskProgression({ progressCurrent: 0n, progressMax: 0n })),
    );
  }

  compact(context: BackupClientContext, sharePath: Buffer): Observable<QueueTaskProgression> {
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
          new QueueTaskProgression({
            compressedFileSize: fileCount.compressedFileSize,
            fileCount: fileCount.count,
            fileSize: fileCount.size,
            progressCurrent: 0n,
            progressMax: 0n,
          }),
      ),
      startWith(new QueueTaskProgression({ progressCurrent: 0n, progressMax: 0n })),
    );
  }

  countRef(context: BackupClientContext): Observable<QueueTaskProgression> {
    const countRef$ = defer(() => this.backupClient.countRef(context));

    return countRef$.pipe(
      startWith(new QueueTaskProgression({ progressCurrent: 0n, progressMax: 0n })),
      mapTo(new QueueTaskProgression()),
      startWith(new QueueTaskProgression({ progressCurrent: 1n, progressMax: 1n })),
    );
  }

  refreshCache(context: BackupClientContext, shares: string[]): Observable<QueueTaskProgression> {
    const refreshCache$ = defer(() => this.backupClient.refreshCache(context, shares));
    return refreshCache$.pipe(
      mapTo(new QueueTaskProgression()),
      startWith(new QueueTaskProgression({ progressCurrent: 0n, progressMax: 0n })),
      startWith(new QueueTaskProgression({ progressCurrent: 1n, progressMax: 1n })),
    );
  }

  close(context: BackupClientContext): void {
    this.backupClient.close(context);
  }
}
