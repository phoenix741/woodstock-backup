import { InternalServerErrorException } from '@nestjs/common';
import { Observable } from 'rxjs';

import { Backup } from '../backups/backup.dto';
import { HostConfiguration } from '../hosts/host-configuration.dto';
import { BackupLogger } from '../logger/BackupLogger.logger';
import { BackupProgression } from '../operation/interfaces/options';
import { pick } from '../utils/lodash';
import { BackupState, BackupSubTask, BackupTask, TaskProgression } from './tasks.dto';

export class InternalBackupSubTask implements BackupSubTask {
  constructor(
    public readonly context: string,
    public readonly description: string,
    public readonly failable: boolean,
    public readonly progress: boolean,
    public readonly command: (
      task: BackupTask,
      subtask: BackupSubTask,
      backupLogger: BackupLogger,
    ) => Observable<BackupProgression | void>,

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
  }

  addSubtask(...subtasks: InternalBackupSubTask[]) {
    for (const subtask of subtasks) {
      this.subtasks.push(subtask);
    }
  }

  start() {
    this.startDate = new Date().getTime();
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
      'previousNumber',
      'ip',
      'startDate',
      'subtasks',
      'state',
      'progression',
      'complete',
    );
  }

  toBackup(): Backup {
    const endDate = new Date().getTime();
    return {
      number: this.number,
      complete: this.complete,

      startDate: this.startDate,
      endDate: this.complete ? endDate : undefined,

      fileCount: this.progression.fileCount,
      newFileCount: this.progression.newFileCount,
      existingFileCount: this.progression.fileCount - this.progression.newFileCount,

      fileSize: this.progression.fileSize,
      existingFileSize: this.progression.fileSize - this.progression.newFileSize,
      newFileSize: this.progression.newFileSize,

      speed: this.progression.newFileSize / (endDate - this.startDate),
    };
  }
}
