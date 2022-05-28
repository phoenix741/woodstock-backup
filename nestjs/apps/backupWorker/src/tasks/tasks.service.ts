import { Injectable, InternalServerErrorException } from '@nestjs/common';
import { BackupLogger, BackupsService, BackupState, Operation, TaskProgression } from '@woodstock/shared';
import * as mkdirp from 'mkdirp';
import { defer, from, Observable, of, throwError } from 'rxjs';
import { catchError, concatMap, map, startWith, switchMap, tap } from 'rxjs/operators';
import { BackupClientGrpc, BackupsGrpcContext } from '../backups/backup-client-grpc.class';
import { BackupClientProgress } from '../backups/backup-client-progress.service';
import { InternalBackupSubTask, InternalBackupTask } from './tasks.class';

@Injectable()
export class TasksService {
  constructor(
    private backupsService: BackupsService,
    private backupsClient: BackupClientProgress,
    private backupGrpcClient: BackupClientGrpc,
  ) {}

  prepareBackup(task: InternalBackupTask): void {
    // Step 1: Clone previous backup
    task.addSubtask(
      new InternalBackupSubTask('storage', `Initialisation`, false, false, () =>
        defer(async () => {
          await mkdirp(this.backupsService.getDestinationDirectory(task.host, task.number));
          await this.backupsService.cloneBackup(task.host, task.previousNumber, task.number);
          return new TaskProgression({ percent: 100 });
        }),
      ),
    );

    // Step 2: Authenticate
    task.addSubtask(
      new InternalBackupSubTask(
        'storage',
        `Authentification to ${task.host} (${task.ip})`,
        false,
        false,
        (connection, _, _2, logger) => this.backupsClient.authenticate(connection, logger),
      ),
    );

    // Step 4: Launch all operation
    task.addSubtask(...this.createSubtask(task.config.operations?.tasks || []));

    // Step 5: Même chose mais avec les posts tasks (but always run)
    task.addSubtask(...this.createSubtask(task.config.operations?.finalizeTasks || [], true));
  }

  private createSubtask(tasks: Operation[], finalize = false): InternalBackupSubTask[] {
    return tasks
      .map((task) => {
        switch (task.name) {
          case 'ExecuteCommand':
            return new InternalBackupSubTask(
              'command',
              `Execute command ${task.command}`,
              finalize,
              false,
              (connection) => this.backupsClient.executeCommand(connection, task.command),
            );
          case 'Backup':
            return [
              new InternalBackupSubTask('cache', `Refresh cache`, false, false, (connection) => {
                return this.backupsClient.refreshCache(
                  connection,
                  task.shares.map((share) => share.name),
                );
              }),
              ...task.shares.map(
                (share) =>
                  new InternalBackupSubTask(
                    'backup',
                    `Get filelist ${share.name}`,
                    finalize,
                    true,
                    (connection, host) => {
                      const includes = [...(share.includes || []), ...(task.includes || [])].map((s) => Buffer.from(s));
                      const excludes = [...(share.excludes || []), ...(task.excludes || [])].map((s) => Buffer.from(s));
                      const sharePath = Buffer.from(share.name);

                      if (!host.ip) {
                        return throwError(() => new Error(`Can't backup host ${host.host}, can't find the IP.`));
                      }

                      return this.backupsClient.getFileList(connection, { includes, excludes, sharePath });
                    },
                  ),
              ),
              ...task.shares.map(
                (share) =>
                  new InternalBackupSubTask('backup', `Backup ${share.name}`, finalize, true, (connection, host) => {
                    const includes = [...(share.includes || []), ...(task.includes || [])].map((s) => Buffer.from(s));
                    const excludes = [...(share.excludes || []), ...(task.excludes || [])].map((s) => Buffer.from(s));
                    const sharePath = Buffer.from(share.name);

                    if (!host.ip) {
                      return throwError(() => new Error(`Can't backup host ${host.host}, can't find the IP.`));
                    }

                    return this.backupsClient.createBackup(connection, { includes, excludes, sharePath });
                  }),
              ),
              ...task.shares.map(
                (share) =>
                  new InternalBackupSubTask('backup', `Compact ${share.name}`, finalize, false, (connection, host) => {
                    const sharePath = Buffer.from(share.name);

                    if (!host.ip) {
                      return throwError(() => new Error(`Can't backup host ${host.host}, can't find the IP.`));
                    }

                    return this.backupsClient.compact(connection, sharePath);
                  }),
              ),
              new InternalBackupSubTask('backup', `Ref count`, false, false, (connection) => {
                return this.backupsClient.countRef(connection);
              }),
            ];
        }
      })
      .flat();
  }

  launchBackup(logger: BackupLogger, task: InternalBackupTask): Observable<InternalBackupTask> {
    task.start();

    logger.log(`Start Backup with state ${task.state}`, 'tasks');
    if (!task.ip) {
      throw new InternalServerErrorException(`Missing ip on host ${task.host}`);
    }

    return from(this.backupGrpcClient.createConnection(task.ip, task.host, task.number)).pipe(
      switchMap((connection) => {
        return from(task.subtasks).pipe(
          concatMap((subtask) => {
            logger.log(
              `Launch subtask ${subtask.description} if ${task.state} is not failed or ${subtask.finalize} is true`,
              'tasks',
            );
            if (![BackupState.FAILED, BackupState.ABORTED].includes(task.state) || subtask.finalize) {
              return this.launchTask(connection, logger, task, subtask);
            } else {
              subtask.state = BackupState.ABORTED;
              return of(task);
            }
          }),
          tap({
            complete: () => {
              this.backupsClient.close(connection);
              logger.log(`End Backup with state ${task.state}`, 'tasks');
              if ([BackupState.FAILED, BackupState.ABORTED].includes(task.state)) {
                throw new Error(`Backup of ${task.host} have been failed`);
              }
            },
          }),
        );
      }),
    );
  }

  private launchTask(
    connection: BackupsGrpcContext,
    logger: BackupLogger,
    task: InternalBackupTask,
    subtask: InternalBackupSubTask,
  ): Observable<InternalBackupTask> {
    logger.log(`Start Subtask ${subtask.description}`, subtask.context);
    subtask.progression = subtask.progression || new TaskProgression();
    subtask.state = BackupState.RUNNING;
    return subtask.command(connection, task, subtask, logger).pipe(
      startWith(subtask.progression),
      map((progression) => {
        subtask.progression = progression || subtask.progression;
        return task;
      }),
      catchError((err) => {
        logger.error(`${err.message}`, err.stack, subtask.context);
        subtask.state = BackupState.FAILED;
        return of(task);
      }),
      tap({
        complete: () => {
          if (![BackupState.ABORTED, BackupState.FAILED].includes(subtask.state)) {
            subtask.progression = subtask.progression || new TaskProgression();
            // subtask.progression.progressMax = subtask.progression.progressMax || 1n;
            // subtask.progression.progressCurrent = subtask.progression.progressMax;
            subtask.state = BackupState.SUCCESS;
          }
          logger.log(`End Subtask with state ${subtask.state}`, subtask.context);
          return task;
        },
      }),
    );
  }
}
