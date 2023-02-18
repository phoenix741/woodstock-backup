import { Injectable, InternalServerErrorException, LoggerService } from '@nestjs/common';
import { BackupOperation, BackupsService, bigIntMax, ExecuteCommandOperation, JobService } from '@woodstock/shared';
import {
  BackupContext,
  BackupShareContext,
  CommandContext,
  GroupContext,
  JobBackupData,
  RefreshCacheContext,
  BackupNameTask,
} from '@woodstock/shared/backuping/backuping.model';
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
import { BackupClientGrpc } from '../backups/backup-client-grpc.class';
import { BackupClientProgress } from '../backups/backup-client-progress.service';

@Injectable()
export class BackupTasksService {
  constructor(
    private backupsService: BackupsService,
    private backupsClient: BackupClientProgress,
    private backupGrpcClient: BackupClientGrpc,
    private jobService: JobService,
    private queueTasksService: QueueTasksService,
  ) {}

  #createCommands(commandOperations: ExecuteCommandOperation[], priority = QueueTaskPriority.PRE_PROCESSING) {
    const commandsGroup = new QueueGroupTasks(
      priority === QueueTaskPriority.PRE_PROCESSING
        ? BackupNameTask.PRE_COMMAND_TASK
        : BackupNameTask.POST_COMMAND_TASK,
      { description: 'commands' } satisfies GroupContext,
    );

    for (const operation of commandOperations) {
      commandsGroup.add(
        new QueueSubTask(
          BackupNameTask.COMMAND_TASK,
          { command: operation.command } satisfies CommandContext,
          priority,
        ),
      );
    }

    return commandsGroup;
  }

  #createBackupTask(backup: BackupOperation) {
    const backupGroup = new QueueGroupTasks(BackupNameTask.BACKUP_TASK, {} satisfies GroupContext);

    backupGroup.add(
      new QueueSubTask(
        BackupNameTask.REFRESH_CACHE_TASK,
        { shares: backup.shares.map((share) => share.name) } satisfies RefreshCacheContext,
        QueueTaskPriority.PRE_PROCESSING,
      ),
    );

    for (const share of backup.shares) {
      const includes = [...(share.includes || []), ...(backup.includes || [])].map((s) => Buffer.from(s));
      const excludes = [...(share.excludes || []), ...(backup.excludes || [])].map((s) => Buffer.from(s));
      const sharePath = Buffer.from(share.name);

      backupGroup.add(
        new QueueGroupTasks(BackupNameTask.GROUP_SHARE_TASK, {
          includes,
          excludes,
          sharePath,
        } satisfies BackupShareContext)
          .add(
            new QueueSubTask(
              BackupNameTask.FILELIST_TASK,
              { includes, excludes, sharePath } satisfies BackupShareContext,
              QueueTaskPriority.PROCESSING,
            ),
          )
          .add(
            new QueueSubTask(
              BackupNameTask.CHUNKS_TASK,
              { includes, excludes, sharePath } satisfies BackupShareContext,
              QueueTaskPriority.PROCESSING,
            ),
          )
          .add(
            new QueueSubTask(
              BackupNameTask.COMPACT_TASK,
              { includes, excludes, sharePath } satisfies BackupShareContext,
              QueueTaskPriority.FINALISATION,
            ),
          ),
      );
    }

    return backupGroup;
  }

  #createGlobalContext(job: Job<JobBackupData>, clientLogger: LoggerService, logger: LoggerService) {
    const connection = this.backupGrpcClient.createContext(job.data.ip, job.data.host, job.data.number ?? 0);
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

      await this.backupGrpcClient.createConnection(gc.globalContext.connection);
      await this.backupsClient.authenticate(
        gc.globalContext.connection,
        logger,
        gc.globalContext.clientLogger,
        gc.globalContext.config?.password,
      );
    });

    globalContext.commands.set(BackupNameTask.COMMAND_TASK, (gc, lc: CommandContext) =>
      this.backupsClient.executeCommand(gc.globalContext.connection, lc.command),
    );
    globalContext.commands.set(BackupNameTask.REFRESH_CACHE_TASK, (gc, lc: RefreshCacheContext) =>
      this.backupsClient.refreshCache(gc.globalContext.connection, lc.shares),
    );
    globalContext.commands.set(BackupNameTask.FILELIST_TASK, (gc, lc: BackupShareContext) =>
      this.backupsClient.getFileList(gc.globalContext.connection, lc),
    );
    globalContext.commands.set(BackupNameTask.CHUNKS_TASK, (gc, lc: BackupShareContext) =>
      this.backupsClient.createBackup(gc.globalContext.connection, lc),
    );
    globalContext.commands.set(BackupNameTask.COMPACT_TASK, (gc, lc: BackupShareContext) =>
      this.backupsClient.compact(gc.globalContext.connection, lc.sharePath),
    );
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

    const task = new QueueTasks('GLOBAL', {})
      .add(
        new QueueGroupTasks(BackupNameTask.GROUP_INIT_TASK, { description: 'initialisation' } satisfies GroupContext)
          .add(new QueueSubTask(BackupNameTask.CONNECTION_TASK, {}, QueueTaskPriority.INITIALISATION))
          .add(new QueueSubTask(BackupNameTask.INIT_DIRECTORY_TASK, {}, QueueTaskPriority.INITIALISATION)),
      )
      .add(this.#createCommands(config?.operations?.preCommands || [], QueueTaskPriority.PRE_PROCESSING))
      .add(this.#createBackupTask(config?.operations?.operation || { shares: [] }))
      .add(this.#createCommands(config?.operations?.preCommands || [], QueueTaskPriority.PROCESSING))
      .add(
        new QueueGroupTasks(BackupNameTask.GROUP_END_TASK, { description: 'finalisation' } satisfies GroupContext)
          .add(new QueueSubTask(BackupNameTask.CLOSE_CONNECTION_TASK, {}, QueueTaskPriority.POST_PROCESSING))
          .add(new QueueSubTask(BackupNameTask.REFCNT_HOST_TASK, {}, QueueTaskPriority.FINALISATION))
          .add(new QueueSubTask(BackupNameTask.REFCNT_POOL_TASK, {}, QueueTaskPriority.FINALISATION)),
      );

    return new QueueTasksInformations(task, this.#createGlobalContext(job, clientLogger, logger));
  }

  launchBackupTask(informations: QueueTasksInformations<BackupContext>) {
    const { context, tasks } = informations;

    return this.queueTasksService.executeTasks(tasks, context);
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

      speed: Number(progression.newFileSize / BigInt(endDate - jobData.startDate)),
    };
  }
}
