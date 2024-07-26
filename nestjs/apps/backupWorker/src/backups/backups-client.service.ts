import { Injectable } from '@nestjs/common';
import { ApplicationConfigService } from '@woodstock/shared';
import {
  JsBackupProgression,
  WoodstockBackupClient,
  WoodstockBackupCommandReply,
  WoodstockBackupShare,
} from '@woodstock/shared-rs';
import { Observable } from 'rxjs';

@Injectable()
export class BackupsClientService {
  constructor(private applicationConfig: ApplicationConfigService) {}

  createClient(hostname: string, ip: string, backupNumber: number): Promise<WoodstockBackupClient> {
    const context = this.applicationConfig.context;
    return WoodstockBackupClient.createClient(hostname, ip, backupNumber, context);
  }

  authenticate(context: WoodstockBackupClient, password: string): Promise<void> {
    return context.authenticate(password);
  }

  createBackupDirectory(context: WoodstockBackupClient, shares: Array<string>): Promise<void> {
    return context.createBackupDirectory(shares);
  }

  executeCommand(context: WoodstockBackupClient, command: string): Promise<WoodstockBackupCommandReply> {
    return context.executeCommand(command);
  }

  uploadFileList(context: WoodstockBackupClient, shares: Array<string>): Promise<void> {
    return context.uploadFileList(shares);
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

  compact(context: WoodstockBackupClient, sharePath: string): Promise<void> {
    return context.compact(sharePath);
  }

  countReferences(context: WoodstockBackupClient): Promise<void> {
    return context.countReferences();
  }

  saveBackup(context: WoodstockBackupClient, completed: boolean): Promise<void> {
    return context.saveBackup(completed);
  }

  close(context: WoodstockBackupClient): Promise<void> {
    return context.close();
  }
}
