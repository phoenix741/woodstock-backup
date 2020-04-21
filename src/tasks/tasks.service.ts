import { Injectable } from '@nestjs/common';
import { join } from 'path';

import { HostConfig, Operation } from '../hosts/host-config.dto';
import { BackupLogger } from '../logger/BackupLogger.logger';
import { ResolveService } from '../network/resolve';
import { ExecuteCommandService } from '../operation/execute-command.service';
import { RSyncCommandService } from '../operation/rsync-command.service';
import { BtrfsService } from '../storage/btrfs/btrfs.service';
import { CallbackTaskChangeFn, InternalBackupSubTask, InternalBackupTask } from './tasks.class';
import { BackupState, TaskProgression } from './tasks.dto';

@Injectable()
export class TasksService {
  constructor(
    public resolveService: ResolveService,
    public btrfsService: BtrfsService,
    public executeCommandService: ExecuteCommandService,
    public rsyncCommandService: RSyncCommandService,
  ) {}

  addSubTasks(task: InternalBackupTask) {
    // Step 1: Clone previous backup
    task.addSubtask(
      new InternalBackupSubTask(
        'storage',
        'Create the destination directory',
        true,
        false,
        async task =>
          task.destinationDirectory &&
          this.btrfsService.createSnapshot(task.destinationDirectory, task.previousDirectory),
      ),
    );

    // Step 2: Launch all operation
    for (const operation of task.config.operations.tasks || []) {
      task.addSubtask(...this.createTaskFromOperation(task.config, operation));
    }

    // Step 3: Même chose mais avec les posts tasks (but always run)
    for (const operation of task.config.operations.finalizeTasks || []) {
      task.addSubtask(...this.createTaskFromOperation(task.config, operation, false));
    }

    // Step 4: Mark storage as readonly if complete
    task.addSubtask(
      new InternalBackupSubTask(
        'storage',
        'Mark as readonly',
        true,
        false,
        async task => task.destinationDirectory && this.btrfsService.markReadOnly(task.destinationDirectory),
      ),
    );
  }

  async launchBackup(logger: BackupLogger, task: InternalBackupTask, taskChanged: CallbackTaskChangeFn) {
    task.start();
    taskChanged(task);

    for (const subtask of task.subtasks) {
      if ([BackupState.FAILED, BackupState.ABORTED].includes(task.state) && subtask.failable) {
        taskChanged(task);
        continue;
      }

      subtask.state = BackupState.RUNNING;
      taskChanged(task);

      try {
        subtask.progression = subtask.progression || new TaskProgression();
        await subtask.command(
          task,
          subtask,
          progression => {
            subtask.progression = progression || subtask.progression;
            taskChanged(task);
          },
          logger,
        );
        subtask.progression.percent = 100;
        subtask.state = BackupState.SUCCESS;

        taskChanged(task);
      } catch (err) {
        logger.error(err.message, err.stack, subtask.context);
        subtask.state = BackupState.FAILED;
        taskChanged(task);
      }
    }

    taskChanged(task);
    if (task.state !== BackupState.SUCCESS) {
      throw new Error(`Backup of ${task.host} have been failed`);
    }
  }

  private createTaskFromOperation(config: HostConfig, operation: Operation, failable = true): InternalBackupSubTask[] {
    switch (operation.name) {
      case 'ExecuteCommand':
        return [
          new InternalBackupSubTask(
            operation.command,
            `Execute command ${operation.command}`,
            failable,
            false,
            async (hist, task, callbackProgress, backupLogger) =>
              await this.executeCommandService.execute(operation, {
                host: config.name,
                context: operation.command,
                callbackProgress,
                backupLogger,
              }),
          ),
        ];
      case 'RSyncBackup':
        return operation.share.map(share => {
          return new InternalBackupSubTask(
            share.name,
            `Execute backup for share ${share.name}`,
            failable,
            true,
            async (host, task, callbackProgress, backupLogger) => {
              const includes = [...(share.includes || []), ...(operation.includes || [])];
              const excludes = [...(share.excludes || []), ...(operation.excludes || [])];

              if (!host.ip) {
                throw new Error(`Can't backup host ${host.host}, can't find the IP.`);
              }
              if (!host.destinationDirectory) {
                throw new Error(`Can't backup host ${host.host}, can't find where to put the backup.`);
              }

              await this.rsyncCommandService.backup(host.ip, share.name, join(host.destinationDirectory, share.name), {
                host: config.name,
                context: share.name,

                rsync: true,
                username: 'root',
                includes,
                excludes,

                callbackProgress,
                backupLogger,
              });
            },
          );
        });
    }
  }
}
