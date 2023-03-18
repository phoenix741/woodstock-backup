import { Injectable, InternalServerErrorException, LoggerService } from '@nestjs/common';
import { BackupOperation, BackupsService, bigIntMax, ExecuteCommandOperation, JobService } from '@woodstock/shared';
import { BackupContext, BackupNameTask, JobBackupData } from '@woodstock/shared/backuping/backuping.model';
import {
  QueueGroupTasks,
  QueueSubTask,
  QueueTaskContext,
  QueueTaskPriority,
  QueueTasks,
  QueueTasksInformations,
  QUEUE_TASK_SUCCESS_STATE,
} from '@woodstock/shared/tasks/queue-tasks.model';
import { QueueTasksService } from '@woodstock/shared/tasks/queue-tasks.service';
import { Job } from 'bullmq';
import mkdirp from 'mkdirp';
import { RedlockAbortSignal } from 'redlock';
import { BackupClientProgress } from '../backups/backup-client-progress.service';

@Injectable()
export class BackupTasksService {
  constructor(
    private backupsService: BackupsService,
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
      const includes = [...(share.includes || []), ...(backup.includes || [])].map((s) => Buffer.from(s));
      const excludes = [...(share.excludes || []), ...(backup.excludes || [])].map((s) => Buffer.from(s));
      const sharePath = Buffer.from(share.name);

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

  #createGlobalContext(job: Job<JobBackupData>, clientLogger: LoggerService, logger: LoggerService) {
    const connection = this.backupsClient.createContext(
      job.data.ip,
      job.data.host,
      job.data.number ?? 0,
      job.data.pathPrefix,
    );
    const globalContext = new QueueTaskContext(new BackupContext(job.data, clientLogger, connection), logger);

    globalContext.commands.set(BackupNameTask.INIT_DIRECTORY_TASK, async (gc) => {
      await mkdirp(this.backupsService.getDestinationDirectory(gc.globalContext.host, gc.globalContext.number));
      await this.backupsService.cloneBackup(
        gc.globalContext.host,
        gc.globalContext.previousNumber,
        gc.globalContext.number,
      );
    });
    globalContext.commands.set(BackupNameTask.CONNECTION_TASK, async (gc) => {
      if (!gc.globalContext.config?.password) {
        throw new InternalServerErrorException('No password provided');
      }

      await this.backupsClient.createConnection(gc.globalContext.connection);
      await this.backupsClient.authenticate(
        gc.globalContext.connection,
        logger,
        gc.globalContext.clientLogger,
        gc.globalContext.config?.password,
      );
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
      return this.backupsClient.refreshCache(gc.globalContext.connection, lc.shares);
    });
    globalContext.commands.set(BackupNameTask.FILELIST_TASK, (gc, { includes, excludes, sharePath }) => {
      if (!includes || !excludes || !sharePath) {
        throw new InternalServerErrorException('No includes, excludes or sharePath provided');
      }

      return this.backupsClient.getFileList(gc.globalContext.connection, { includes, excludes, sharePath });
    });
    globalContext.commands.set(BackupNameTask.CHUNKS_TASK, (gc, { includes, excludes, sharePath }) => {
      if (!includes || !excludes || !sharePath) {
        throw new InternalServerErrorException('No includes, excludes or sharePath provided');
      }

      return this.backupsClient.createBackup(gc.globalContext.connection, { includes, excludes, sharePath });
    });
    globalContext.commands.set(BackupNameTask.COMPACT_TASK, (gc, lc) => {
      if (!lc.sharePath) {
        throw new InternalServerErrorException('No includes, excludes or sharePath provided');
      }

      return this.backupsClient.compact(gc.globalContext.connection, lc.sharePath);
    });
    globalContext.commands.set(BackupNameTask.CLOSE_CONNECTION_TASK, async (gc) => {
      this.backupsClient.close(gc.globalContext.connection);
    });
    globalContext.commands.set(BackupNameTask.REFCNT_HOST_TASK, (gc) => {
      if (!gc.globalContext.connection) {
        throw new InternalServerErrorException('No connection to backup server');
      }

      return this.backupsClient.countRef(gc.globalContext.connection);
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

    return globalContext;
  }

  prepareBackupTask(
    job: Job<JobBackupData>,
    clientLogger: LoggerService,
    logger: LoggerService,
  ): QueueTasksInformations<BackupContext> {
    const config = job.data.config;

    const task = new QueueTasks('GLOBAL')
      .add(
        new QueueGroupTasks(BackupNameTask.GROUP_INIT_TASK)
          .add(new QueueSubTask(BackupNameTask.CONNECTION_TASK, {}, QueueTaskPriority.INITIALISATION))
          .add(new QueueSubTask(BackupNameTask.INIT_DIRECTORY_TASK, {}, QueueTaskPriority.INITIALISATION))
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
          .add(new QueueSubTask(BackupNameTask.REFCNT_POOL_TASK, {}, QueueTaskPriority.FINALISATION)),
      );

    return new QueueTasksInformations(task, this.#createGlobalContext(job, clientLogger, logger));
  }

  launchBackupTask(
    job: Job<JobBackupData>,
    informations: QueueTasksInformations<BackupContext>,
    signal: RedlockAbortSignal,
  ) {
    return this.queueTasksService.executeTasksFromJob(job, informations, async (informations) => {
      await this.backupsService.addOrReplaceBackup(job.data.host, this.toBackup(informations));

      if (signal.aborted) {
        throw signal.error;
      }
    });
  }

  serializeBackupTask(tasks: QueueTasks): object {
    return this.queueTasksService.serializeBackupTask(tasks);
  }

  deserializeBackupTask(data: object): QueueTasks {
    return this.queueTasksService.deserializeBackupTask(data);
  }

  toBackup(informations: QueueTasksInformations<BackupContext>) {
    const progression = informations.tasks.progression;
    const jobData = informations.context.globalContext;

    const endDate = new Date().getTime();

    return {
      number: jobData.number,
      complete: QUEUE_TASK_SUCCESS_STATE.includes(informations.tasks.state),

      startDate: jobData.startDate,
      endDate,

      fileCount: progression.fileCount,
      newFileCount: progression.newFileCount,
      existingFileCount: Math.max(progression.fileCount - progression.newFileCount, 0),

      fileSize: progression.fileSize,
      newFileSize: progression.newFileSize,
      existingFileSize: bigIntMax(progression.fileSize - progression.newFileSize, 0n),

      compressedFileSize: progression.compressedFileSize,
      existingCompressedFileSize: bigIntMax(progression.compressedFileSize - progression.newCompressedFileSize, 0n),
      newCompressedFileSize: progression.newCompressedFileSize,

      speed: Number(progression.newFileSize / BigInt(endDate - jobData.startDate)) * 1000,
    };
  }
}
