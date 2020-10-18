import { Injectable } from '@nestjs/common';
import { from, Observable, of, throwError } from 'rxjs';
import { catchError, concatMap, map, startWith, tap } from 'rxjs/operators';

import { Operation } from '../hosts/host-configuration.dto';
import { BackupLogger } from '../logger/BackupLogger.logger';
import { ResolveService } from '../network/resolve';
import { ExecuteCommandService } from '../operation/execute-command.service';
import { RSyncCommandService } from '../operation/rsync-command.service';
import { BtrfsService } from '../storage/btrfs/btrfs.service';
import { pick } from '../utils/lodash';
import { InternalBackupSubTask, InternalBackupTask } from './tasks.class';
import { BackupState, TaskProgression } from './tasks.dto';

@Injectable()
export class TasksService {
  constructor(
    public resolveService: ResolveService,
    public btrfsService: BtrfsService,
    public executeCommandService: ExecuteCommandService,
    public rsyncCommandService: RSyncCommandService,
  ) {}

  addSubTasks(task: InternalBackupTask): void {
    // Step 1: Clone previous backup
    task.addSubtask(
      new InternalBackupSubTask('storage', 'Create the destination directory', true, false, (task) => {
        return from(
          this.btrfsService.createSnapshot({
            hostname: task.host,
            destBackupNumber: task.number,
            srcBackupNumber: task.previousNumber,
          }),
        );
      }),
    );

    // Step 2: Launch all operation
    for (const operation of task.config.operations?.tasks || []) {
      task.addSubtask(...this.createTaskFromOperation(operation));
    }

    // Step 3: Même chose mais avec les posts tasks (but always run)
    for (const operation of task.config.operations?.finalizeTasks || []) {
      task.addSubtask(...this.createTaskFromOperation(operation, false));
    }

    // Step 4: Mark storage as readonly if complete
    task.addSubtask(
      new InternalBackupSubTask('storage', 'Mark as readonly', true, false, (task) => {
        return from(
          this.btrfsService.markReadOnly({
            hostname: task.host,
            destBackupNumber: task.number,
            srcBackupNumber: task.previousNumber,
          }),
        );
      }),
    );
  }

  launchBackup(logger: BackupLogger, task: InternalBackupTask): Observable<InternalBackupTask> {
    task.start();

    logger.log(`[${task.host}] - Start Backup with state ${task.state}`);
    return from(task.subtasks).pipe(
      concatMap((subtask) => {
        logger.log(`[${task.host}] - Launch subtask if ${task.state} is not failed or ${subtask.failable} is true`);
        if (![BackupState.FAILED, BackupState.ABORTED].includes(task.state) || !subtask.failable) {
          return this.launchTask(logger, task, subtask);
        } else {
          subtask.state = BackupState.ABORTED;
          return of(task);
        }
      }),
      tap(undefined, undefined, () => {
        logger.log(`[${task.host}] - End Backup with state ${task.state}`);
        if ([BackupState.FAILED, BackupState.ABORTED].includes(task.state)) {
          throw new Error(`Backup of ${task.host} have been failed`);
        }
      }),
    );
  }

  private launchTask(
    logger: BackupLogger,
    task: InternalBackupTask,
    subtask: InternalBackupSubTask,
  ): Observable<InternalBackupTask> {
    logger.log(`[${task.host}][${subtask.context}] - Start Subtask`, subtask.context);
    subtask.progression = subtask.progression || new TaskProgression();
    subtask.state = BackupState.RUNNING;
    return subtask.command(task, subtask, logger).pipe(
      startWith(subtask.progression),
      map((progression) => {
        subtask.progression = progression || subtask.progression;
        return task;
      }),
      catchError((err) => {
        logger.error(`[${task.host}][${subtask.context}] - ${err.message}`, err.stack, subtask.context);
        subtask.state = BackupState.FAILED;
        return of(task);
      }),
      tap(undefined, undefined, () => {
        if (![BackupState.ABORTED, BackupState.FAILED].includes(subtask.state)) {
          subtask.progression = subtask.progression || new TaskProgression();
          subtask.progression.percent = 100;
          subtask.state = BackupState.SUCCESS;
        }
        logger.log(`[${task.host}][${subtask.context}] - End Subtask with state ${subtask.state}`, subtask.context);
        return task;
      }),
    );
  }

  private createTaskFromOperation(operation: Operation, failable = true): InternalBackupSubTask[] {
    switch (operation.name) {
      case 'ExecuteCommand':
        return [
          new InternalBackupSubTask(
            operation.command,
            `Execute command ${operation.command}`,
            failable,
            false,
            (_, __, backupLogger) =>
              this.executeCommandService.execute(operation, {
                context: operation.command,
                backupLogger,
              }),
          ),
        ];
      case 'RSyncBackup':
      case 'RSyncdBackup':
        return operation.share.map((share) => {
          return new InternalBackupSubTask(
            share.name,
            `Execute backup for share ${share.name}`,
            failable,
            true,
            (host, _, backupLogger) => {
              const includes = [...(share.includes || []), ...(operation.includes || [])];
              const excludes = [...(share.excludes || []), ...(operation.excludes || [])];

              if (!host.ip && !host.config?.isLocal) {
                return throwError(new Error(`Can't backup host ${host.host}, can't find the IP.`));
              }

              return this.rsyncCommandService.backup(
                {
                  hostname: host.host,
                  ip: host.ip,
                  srcBackupNumber: host.previousNumber,
                  destBackupNumber: host.number,
                },
                share.name,
                {
                  context: share.name,

                  rsync: operation.name === 'RSyncBackup',
                  ...(operation.name === 'RSyncBackup' ? pick(operation, 'rsyncPath', 'username') : {}),

                  rsyncd: operation.name === 'RSyncdBackup',
                  ...(operation.name === 'RSyncdBackup'
                    ? pick(operation, 'authentification', 'username', 'password')
                    : {}),

                  options: operation.options,

                  includes,
                  excludes,
                  checksum: share.checksum,

                  pathPrefix: share.pathPrefix,

                  backupLogger,
                },
              );
            },
          );
        });
    }
  }
}
