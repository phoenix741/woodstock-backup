import { InternalServerErrorException } from '@nestjs/common';
import {
  Backup,
  BackupLogger,
  BackupState,
  BackupSubTask,
  BackupTask,
  bigIntMax,
  HostConfiguration,
  pick,
  TaskProgression,
} from '@woodstock/shared';
import { Observable } from 'rxjs';
import { BackupsGrpcContext } from '../backups/backup-client-grpc.class.js';

export class InternalBackupSubTask implements BackupSubTask {
  constructor(
    public readonly context: string,
    public readonly description: string,
    public readonly finalize: boolean,
    public readonly progress: boolean,
    public readonly command: (
      connection: BackupsGrpcContext,
      task: BackupTask,
      subtask: BackupSubTask,
      backupLogger: BackupLogger,
    ) => Observable<TaskProgression>,
    public state: BackupState = BackupState.WAITING,
    public progression?: TaskProgression,
  ) {}
}

export class InternalBackupTask implements BackupTask {
  public readonly host: string;
  public readonly config: HostConfiguration;
  public readonly number: number;
  public readonly previousNumber?: number;
  public readonly ip?: string;

  public startDate: number = new Date().getTime();
  public originalStartDate?: number;
  public subtasks: InternalBackupSubTask[] = [];

  constructor(original: BackupTask) {
    if (!original.config || original.number === undefined || (!original.ip && !original.config.isLocal)) {
      throw new InternalServerErrorException(`Initialisation of backup failed.`);
    }

    this.host = original.host;
    this.config = original.config;
    this.previousNumber = original.previousNumber;
    this.number = original.number;
    this.ip = original.ip;
    this.originalStartDate = original.originalStartDate;
  }

  addSubtask(...subtasks: InternalBackupSubTask[]): void {
    for (const subtask of subtasks) {
      this.subtasks.push(subtask);
    }
  }

  start(): void {
    this.startDate = new Date().getTime();
  }

  get state(): BackupState {
    const subtasks = this.subtasks;

    const running = subtasks.some((subtask) => [BackupState.RUNNING, BackupState.WAITING].includes(subtask.state));
    const failed = subtasks.some((subtask) => [BackupState.FAILED, BackupState.ABORTED].includes(subtask.state));
    if (running) {
      if (failed) {
        return BackupState.ABORTED;
      }

      const started = subtasks.some((subtask) => [BackupState.RUNNING].includes(subtask.state));
      if (started) {
        return BackupState.RUNNING;
      }
      return BackupState.WAITING;
    } else {
      if (failed) {
        return BackupState.FAILED;
      }
      return BackupState.SUCCESS;
    }
  }

  get complete(): boolean {
    return this.state === BackupState.SUCCESS;
  }

  get progression(): TaskProgression {
    const progression = this.subtasks.reduce((acc, subtask) => {
      acc.fileCount += subtask.progression?.fileCount || 0;
      acc.fileSize += subtask.progression?.fileSize || 0n;
      acc.compressedFileSize += subtask.progression?.compressedFileSize || 0n;
      acc.newFileCount += subtask.progression?.newFileCount || 0;
      acc.newFileSize += subtask.progression?.newFileSize || 0n;
      acc.newCompressedFileSize += subtask.progression?.newCompressedFileSize || 0n;
      if (subtask.progress) {
        acc.progressMax += subtask.progression?.progressMax || 0n;
        acc.progressCurrent += subtask.progression?.progressCurrent || 0n;
      }
      acc.speed = subtask.progression?.speed || acc.speed;
      return acc;
    }, new TaskProgression());
    return progression;
  }

  toJSON(): Pick<
    this,
    | 'number'
    | 'host'
    | 'config'
    | 'previousNumber'
    | 'ip'
    | 'startDate'
    | 'originalStartDate'
    | 'subtasks'
    | 'state'
    | 'progression'
    | 'complete'
  > {
    return pick(
      this,
      'host',
      'config',
      'number',
      'previousNumber',
      'ip',
      'startDate',
      'originalStartDate',
      'subtasks',
      'state',
      'progression',
      'complete',
    );
  }

  toBackup(): Backup {
    const endDate = new Date().getTime();
    const calculateEndDate = this.originalStartDate ? this.originalStartDate + (endDate - this.startDate) : endDate;
    return {
      number: this.number,
      complete: this.complete,

      startDate: this.originalStartDate || this.startDate,
      endDate: this.complete ? calculateEndDate : undefined,

      fileCount: this.progression.fileCount,
      newFileCount: this.progression.newFileCount,
      existingFileCount: Math.max(this.progression.fileCount - this.progression.newFileCount, 0),

      fileSize: this.progression.fileSize,
      newFileSize: this.progression.newFileSize,
      existingFileSize: bigIntMax(this.progression.fileSize - this.progression.newFileSize, 0n),

      compressedFileSize: this.progression.compressedFileSize,
      existingCompressedFileSize: bigIntMax(
        this.progression.compressedFileSize - this.progression.newCompressedFileSize,
        0n,
      ),
      newCompressedFileSize: this.progression.newCompressedFileSize,

      speed: Number(this.progression.newFileSize / BigInt(endDate - this.startDate)),
    };
  }
}
