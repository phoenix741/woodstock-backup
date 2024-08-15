import { Injectable, InternalServerErrorException } from '@nestjs/common';
import {
  BackupContext,
  BackupLogger,
  BackupNameTask,
  BackupOperation,
  ExecuteCommandOperation,
  JobBackupData,
  JobService,
  QUEUE_TASK_FAILED_STATE,
  QueueGroupTasks,
  QueueSubTask,
  QueueTaskContext,
  QueueTaskPriority,
  QueueTasks,
  QueueTasksInformations,
  QueueTasksService,
} from '@woodstock/shared';
import { Job } from 'bullmq';
import { BackupClientProgress } from '../backups/backup-client-progress.service';

@Injectable()
export class BackupTasksService {
  constructor(
    private backupsClient: BackupClientProgress,
    private jobService: JobService,
    private queueTasksService: QueueTasksService,
  ) {}

  #createCommands(commandOperations: ExecuteCommandOperation[], priority = QueueTaskPriority.PRE_PROCESSING) {
    const commandsGroup = new QueueGroupTasks(
      priority === QueueTaskPriority.PRE_PROCESSING
        ? BackupNameTask.PRE_COMMAND_TASK
        : BackupNameTask.POST_COMMAND_TASK,
    );

    for (const operation of commandOperations) {
      commandsGroup.add(new QueueSubTask(BackupNameTask.COMMAND_TASK, { command: operation.command }, priority));
    }

    return commandsGroup;
  }

  #createBackupTask(backup: BackupOperation) {
    const backupGroup = new QueueGroupTasks(BackupNameTask.BACKUP_TASK);

    for (const share of backup.shares) {
      const includes = [...(share.includes || []), ...(backup.includes || [])];
      const excludes = [...(share.excludes || []), ...(backup.excludes || [])];
      const sharePath = share.name;

      backupGroup.add(
        new QueueGroupTasks(BackupNameTask.GROUP_SHARE_TASK, {
          includes,
          excludes,
          sharePath,
        })
          .add(
            new QueueSubTask(
              BackupNameTask.FILELIST_TASK,
              { includes, excludes, sharePath },
              QueueTaskPriority.PRE_PROCESSING,
            ),
          )
          .add(
            new QueueSubTask(
              BackupNameTask.CHUNKS_TASK,
              { includes, excludes, sharePath },
              QueueTaskPriority.PROCESSING,
            ),
          )
          .add(
            new QueueSubTask(
              BackupNameTask.COMPACT_TASK,
              { includes, excludes, sharePath },
              QueueTaskPriority.FINALISATION,
            ),
          ),
      );
    }

    return backupGroup;
  }

  async #createGlobalContext(job: Job<JobBackupData>, clientLogger: BackupLogger, logger: BackupLogger) {
    if (job.data.ip === undefined) {
      throw new InternalServerErrorException('No IP provided');
    }

    const connection = await this.backupsClient.createClient(job.data.host, job.data.ip, job.data.number ?? 0);
    const globalContext = new QueueTaskContext(new BackupContext(job.data, clientLogger, connection), logger);

    globalContext.commands.set(BackupNameTask.CONNECTION_TASK, async (gc) => {
      if (!gc.globalContext.config?.password) {
        throw new InternalServerErrorException('No password provided');
      }

      await this.backupsClient.authenticate(gc.globalContext.connection, gc.globalContext.config?.password);
    });
    globalContext.commands.set(BackupNameTask.INIT_DIRECTORY_TASK, (gc, lc) => {
      return this.backupsClient.createBackupDirectory(gc.globalContext.connection, lc.shares ?? []);
    });

    globalContext.commands.set(BackupNameTask.COMMAND_TASK, (gc, lc) => {
      if (!lc.command) {
        throw new InternalServerErrorException('No command provided');
      }

      return this.backupsClient.executeCommand(gc.globalContext.connection, lc.command);
    });
    globalContext.commands.set(BackupNameTask.REFRESH_CACHE_TASK, (gc, lc) => {
      if (!lc.shares) {
        throw new InternalServerErrorException('No shares provided');
      }
      return this.backupsClient.uploadFileList(gc.globalContext.connection, lc.shares);
    });
    globalContext.commands.set(BackupNameTask.FILELIST_TASK, (gc, { includes, excludes, sharePath }) => {
      if (!includes || !excludes || !sharePath) {
        throw new InternalServerErrorException('No includes, excludes or sharePath provided');
      }

      return this.backupsClient.downloadFileList(gc.globalContext.connection, { includes, excludes, sharePath });
    });
    globalContext.commands.set(BackupNameTask.CHUNKS_TASK, (gc, { includes, excludes, sharePath }) => {
      if (!includes || !excludes || !sharePath) {
        throw new InternalServerErrorException('No includes, excludes or sharePath provided');
      }

      return this.backupsClient.createBackup(gc.globalContext.connection, sharePath);
    });
    globalContext.commands.set(BackupNameTask.COMPACT_TASK, (gc, lc) => {
      if (!lc.sharePath) {
        throw new InternalServerErrorException('No includes, excludes or sharePath provided');
      }

      return this.backupsClient.compact(gc.globalContext.connection, lc.sharePath);
    });
    globalContext.commands.set(BackupNameTask.CLOSE_CONNECTION_TASK, async (gc) => {
      gc.globalContext.logSubscriber?.unsubscribe();
      return this.backupsClient.close(gc.globalContext.connection);
    });
    globalContext.commands.set(BackupNameTask.REFCNT_HOST_TASK, (gc) => {
      if (!gc.globalContext.connection) {
        throw new InternalServerErrorException('No connection to backup server');
      }

      return this.backupsClient.countReferences(gc.globalContext.connection);
    });
    globalContext.commands.set(BackupNameTask.REFCNT_POOL_TASK, async () => {
      if (!job.id) {
        throw new InternalServerErrorException("Can't count ref without job id");
      }
      if (job.data.number === undefined || job.data.number === null) {
        throw new InternalServerErrorException("Can't count ref without job number");
      }

      return this.jobService.launchRefcntJob(
        job.id,
        `${job.prefix}:${job.queueName}`,
        job.data.host,
        job.data.number,
        'add_backup',
      );
    });
    globalContext.commands.set(BackupNameTask.SAVE_BACKUP_TASK, (gc, _, isFailing) => {
      return this.backupsClient.saveBackup(gc.globalContext.connection, !isFailing);
    });

    return globalContext;
  }

  async prepareBackupTask(
    job: Job<JobBackupData>,
    clientLogger: BackupLogger,
    logger: BackupLogger,
  ): Promise<QueueTasksInformations<BackupContext>> {
    const config = job.data.config;

    const task = new QueueTasks('GLOBAL')
      .add(
        new QueueGroupTasks(BackupNameTask.GROUP_INIT_TASK)
          .add(new QueueSubTask(BackupNameTask.CONNECTION_TASK, {}, QueueTaskPriority.INITIALISATION))
          .add(
            new QueueSubTask(
              BackupNameTask.INIT_DIRECTORY_TASK,
              { shares: config?.operations?.operation?.shares.map((share) => share.name) || [] },
              QueueTaskPriority.INITIALISATION,
            ),
          )
          .add(
            new QueueSubTask(
              BackupNameTask.REFRESH_CACHE_TASK,
              { shares: config?.operations?.operation?.shares.map((share) => share.name) || [] },
              QueueTaskPriority.PRE_PROCESSING,
            ),
          ),
      )
      .add(this.#createCommands(config?.operations?.preCommands || [], QueueTaskPriority.PRE_PROCESSING))
      .add(...this.#createBackupTask(config?.operations?.operation || { shares: [] }).subtasks) // We flatten the array, and don't create backup group
      .add(this.#createCommands(config?.operations?.postCommands || [], QueueTaskPriority.PROCESSING))
      .add(
        new QueueGroupTasks(BackupNameTask.GROUP_END_TASK)
          .add(new QueueSubTask(BackupNameTask.CLOSE_CONNECTION_TASK, {}, QueueTaskPriority.POST_PROCESSING))
          .add(new QueueSubTask(BackupNameTask.REFCNT_HOST_TASK, {}, QueueTaskPriority.FINALISATION))
          .add(new QueueSubTask(BackupNameTask.REFCNT_POOL_TASK, {}, QueueTaskPriority.FINALISATION))
          .add(new QueueSubTask(BackupNameTask.SAVE_BACKUP_TASK, {}, QueueTaskPriority.FINALISATION)),
      );

    return new QueueTasksInformations(task, await this.#createGlobalContext(job, clientLogger, logger));
  }

  launchBackupTask(job: Job<JobBackupData>, informations: QueueTasksInformations<BackupContext>, signal: AbortSignal) {
    return this.queueTasksService.executeTasksFromJob(job, informations, async () => {
      if (signal.aborted) {
        throw new Error('Aborted task');
      }
    });
  }

  serializeBackupTask(tasks: QueueTasks): object {
    return this.queueTasksService.serializeBackupTask(tasks);
  }

  deserializeBackupTask(data: object): QueueTasks {
    return this.queueTasksService.deserializeBackupTask(data);
  }
}
