import { CACHE_MANAGER } from '@nestjs/cache-manager';
import { Inject, Injectable } from '@nestjs/common';
import { ApplicationConfigService, BackupsService } from '@woodstock/shared';
import {
  JsBackupProgression,
  WoodstockBackupClient,
  WoodstockBackupCommandReply,
  WoodstockBackupShare,
} from '@woodstock/shared-rs';
import { Cache } from 'cache-manager';
import { Observable } from 'rxjs';

@Injectable()
export class BackupsClientService {
  constructor(
    private backupsService: BackupsService,
    private applicationConfig: ApplicationConfigService,
  ) {}

  createClient(hostname: string, ip: string, backupNumber: number): Promise<WoodstockBackupClient> {
    const context = this.applicationConfig.context;
    return WoodstockBackupClient.createClient(hostname, ip, backupNumber, context);
  }

  authenticate(context: WoodstockBackupClient, password: string): Promise<void> {
    return context.authenticate(password);
  }

  async createBackupDirectory(context: WoodstockBackupClient, shares: Array<string>): Promise<void> {
    await context.createBackupDirectory(shares);

    await this.backupsService.invalidateBackup(context.hostname, context.backupNumber);
  }

  executeCommand(context: WoodstockBackupClient, command: string): Promise<WoodstockBackupCommandReply> {
    return context.executeCommand(command);
  }

  async uploadFileList(context: WoodstockBackupClient, shares: Array<string>): Promise<void> {
    await context.uploadFileList(shares);

    await this.backupsService.invalidateBackup(context.hostname, context.backupNumber);
  }

  downloadFileList(
    context: WoodstockBackupClient,
    share: WoodstockBackupShare,
    abort?: AbortSignal,
  ): Observable<JsBackupProgression> {
    return new Observable((observer) => {
      let abortMethod: () => void = () => {};
      const abortHandle = context.downloadFileList(share, (result) => {
        if (result.progress) {
          observer.next(result.progress);
        }

        if (result.error) {
          abort?.removeEventListener('abort', abortMethod);
          observer.error(result.error);
        }

        if (result.complete) {
          abort?.removeEventListener('abort', abortMethod);
          observer.complete();
        }
      });
      abortMethod = () => {
        abortHandle.abort();
      };

      abort?.addEventListener('abort', abortMethod);
    });
  }

  createBackup(
    context: WoodstockBackupClient,
    sharePath: string,
    abort?: AbortSignal,
  ): Observable<JsBackupProgression> {
    return new Observable((observer) => {
      let abortMethod: () => void = () => {};
      const abortHandle = context.createBackup(sharePath, (result) => {
        if (result.progress) {
          observer.next(result.progress);
        }

        if (result.error) {
          abort?.removeEventListener('abort', abortMethod);
          observer.error(result.error);
        }

        if (result.complete) {
          abort?.removeEventListener('abort', abortMethod);
          observer.complete();
        }
      });
      abortMethod = () => {
        abortHandle.abort();
      };

      abort?.addEventListener('abort', abortMethod);
    });
  }

  async compact(context: WoodstockBackupClient, sharePath: string): Promise<void> {
    await context.compact(sharePath);

    await this.backupsService.invalidateBackup(context.hostname, context.backupNumber);
  }

  countReferences(context: WoodstockBackupClient): Promise<void> {
    return context.countReferences();
  }

  async saveBackup(context: WoodstockBackupClient, completed: boolean): Promise<void> {
    await context.saveBackup(completed);

    await this.backupsService.invalidateBackup(context.hostname, context.backupNumber);
  }

  close(context: WoodstockBackupClient): Promise<void> {
    return context.close();
  }
}
