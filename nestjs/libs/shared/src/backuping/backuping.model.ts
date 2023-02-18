import { BadRequestException, LoggerService } from '@nestjs/common';
import { HostConfiguration } from '@woodstock/shared';
import { BackupsGrpcContext } from 'apps/backupWorker/src/backups/backup-client-grpc.class';

export enum BackupNameTask {
  GROUP_INIT_TASK = 'init',
  INIT_DIRECTORY_TASK = 'init-directory',
  CONNECTION_TASK = 'connection',
  REFRESH_CACHE_TASK = 'refreshcache',
  BACKUP_TASK = 'backup',
  PRE_COMMAND_TASK = 'pre-command',
  POST_COMMAND_TASK = 'post-command',
  COMMAND_TASK = 'command',
  GROUP_SHARE_TASK = 'share',
  FILELIST_TASK = 'filelist',
  CHUNKS_TASK = 'chunks',
  COMPACT_TASK = 'compact',
  GROUP_END_TASK = 'end',
  CLOSE_CONNECTION_TASK = 'close-connection',
  REFCNT_HOST_TASK = 'refcnt-host',
  REFCNT_POOL_TASK = 'refcnt-pool',
}

export class JobBackupData {
  host!: string;
  config?: HostConfiguration;
  previousNumber?: number;
  number?: number;
  ip?: string;
  startDate?: number;
  originalStartDate?: number;

  force?: boolean;
}

export class BackupContext {
  host!: string;
  config: HostConfiguration;
  number: number;
  previousNumber?: number;
  ip?: string;

  startDate: number = new Date().getTime();

  connection: BackupsGrpcContext;
  clientLogger: LoggerService;

  constructor(jobData: JobBackupData, clientLogger: LoggerService, connection: BackupsGrpcContext) {
    if (!jobData.config || jobData.number === undefined || (!jobData.ip && !jobData.config.isLocal)) {
      throw new BadRequestException(`Initialisation of backup failed.`);
    }

    this.host = jobData.host;
    this.config = jobData.config;
    this.previousNumber = jobData.previousNumber;
    this.number = jobData.number;
    this.ip = jobData.ip;

    this.clientLogger = clientLogger;
    this.connection = connection;
  }
}

export interface GroupContext {
  description?: string;
}

export interface CommandContext {
  command: string;
}

export interface RefreshCacheContext {
  shares: string[];
}

export interface BackupShareContext {
  includes: Buffer[];
  excludes: Buffer[];
  sharePath: Buffer;
}
