import { Backup } from '../backups/backup.dto';
import { HostConfig } from '../hosts/host-config.dto';
import { CallbackProgressFn } from '../operation/interfaces/options';
import { pick } from '../utils/lodash';
import { BackupState, BackupSubTask, BackupTask, TaskProgression } from './tasks.dto';
import { InternalServerErrorException } from '@nestjs/common';
import { BackupLogger } from '../logger/BackupLogger.logger';

export type CallbackTaskChangeFn = (task: BackupTask) => void;

export class InternalBackupSubTask implements BackupSubTask {
  constructor(
    public readonly context: string,
    public readonly description: string,
    public readonly failable: boolean,
    public readonly progress: boolean,
    public readonly command: (
      task: BackupTask,
      subtask: BackupSubTask,
      progressFn: CallbackProgressFn,
      backupLogger: BackupLogger,
    ) => Promise<any>,

    public state: BackupState = BackupState.WAITING,
    public progression?: TaskProgression,
  ) {}
}

export class InternalBackupTask implements BackupTask {
  public readonly host: string;
  public readonly config: HostConfig;
  public readonly number: number;
  public readonly ip: string;
  public readonly destinationDirectory: string;
  public readonly previousDirectory?: string;

  public startDate: Date = new Date();
  public subtasks: InternalBackupSubTask[] = [];

  constructor(original: BackupTask) {
    if (!original.config || original.number === undefined || !original.ip || !original.destinationDirectory) {
      throw new InternalServerErrorException(`Initialisation of backup failed.`);
    }

    this.host = original.host;
    this.config = original.config;
    this.number = original.number;
    this.ip = original.ip;
    this.destinationDirectory = original.destinationDirectory;
    this.previousDirectory = original.previousDirectory;
  }

  addSubtask(...subtasks: InternalBackupSubTask[]) {
    for (const subtask of subtasks) {
      this.subtasks.push(subtask);
    }
  }

  start() {
    this.startDate = new Date();
  }

  get state() {
    const running = this.subtasks.some(subtask => [BackupState.RUNNING, BackupState.WAITING].includes(subtask.state));
    const failed = this.subtasks.some(subtask => [BackupState.FAILED, BackupState.ABORTED].includes(subtask.state));
    if (running) {
      if (failed) {
        return BackupState.ABORTED;
      }

      const started = this.subtasks.some(subtask => [BackupState.RUNNING].includes(subtask.state));
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

  get complete() {
    return this.state === BackupState.SUCCESS;
  }

  get progression() {
    const subtasks = this.subtasks.filter(subtask => !!subtask.progress);

    const progression = subtasks.reduce((acc, subtask) => {
      acc.fileCount += subtask.progression?.fileCount || 0;
      acc.fileSize += subtask.progression?.fileSize || 0;
      acc.newFileCount += subtask.progression?.newFileCount || 0;
      acc.newFileSize += subtask.progression?.newFileSize || 0;
      acc.percent += subtask.progression?.percent || 0;
      acc.speed = subtask.progression?.speed || acc.speed;
      return acc;
    }, new TaskProgression());
    progression.percent = progression.percent / subtasks.length;
    return progression;
  }

  toJSON() {
    return pick(
      this,
      'host',
      'config',
      'number',
      'ip',
      'destinationDirectory',
      'startDate',
      'subtasks',
      'state',
      'progression',
      'complete',
    );
  }

  toBackup(): Backup {
    const endDate = new Date();
    return {
      number: this.number,
      complete: this.complete,

      startDate: this.startDate,
      endDate,

      fileCount: this.progression.fileCount,
      newFileCount: this.progression.newFileCount,
      existingFileCount: this.progression.fileCount - this.progression.newFileCount,

      fileSize: this.progression.fileSize,
      existingFileSize: this.progression.fileSize - this.progression.newFileSize,
      newFileSize: this.progression.newFileSize,

      speed: this.progression.newFileSize / (endDate.getTime() - this.startDate.getTime()),
    };
  }
}
