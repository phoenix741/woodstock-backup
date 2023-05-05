import { Processor, WorkerHost } from '@nestjs/bullmq';
import { BadRequestException, Logger, NotFoundException } from '@nestjs/common';
import {
  ApplicationConfigService,
  BackupLogger,
  BackupsService,
  FsckService,
  HostsService,
  PoolService,
  RefcntJobData,
  RefCntService,
  ReferenceCount,
} from '@woodstock/shared';
import {
  QueueSubTask,
  QueueTaskContext,
  QueueTaskPriority,
  QueueTaskProgression,
  QueueTasks,
  QueueTasksInformations,
  QueueTasksService,
} from '@woodstock/shared/tasks';
import { JobLogger } from '@woodstock/shared/tasks';
import { Job } from 'bullmq';
import { scan } from 'rxjs';

enum RefcntTaskNameEnum {
  // Add backup to refcnt pool
  ADD_REFCNT_POOL_TASK = 'ADD_REFCNT_POOL_TASK',

  // Remove backup from refcnt pool
  REMOVE_REFCNT_POOL_TASK = 'REMOVE_REFCNT_POOL_TASK',

  // Cleanup refcnt pool
  PREPARE_CLEAN_UNUSED_POOL_TASK = 'PREPARE_CLEAN_UNUSED_POOL_TASK',
  CLEAN_UNUSED_POOL_TASK = 'CLEAN_UNUSED_POOL_TASK',

  // Check and fix refcnt pool
  VERIFY_HOST_BACKUP_REFCNT = 'VERIFY_HOST_BACKUP_REFCNT',
  VERIFY_HOST_REFCNT = 'VERIFY_HOST_REFCNT',
  VERIFY_POOL_REFCNT = 'VERIFY_POOL_REFCNT',

  // Verify checksum
  VERIFY_CHECKSUM = 'VERIFY_CHECKSUM',
}

@Processor('refcnt', {
  concurrency: 1,
  removeOnComplete: {
    // TODO: Configuration
    age: 3600,
  },
  removeOnFail: {
    age: 60 * 60 * 24 * 2,
  },
})
export class RefcntConsumer extends WorkerHost {
  #logger = new Logger(RefcntConsumer.name);

  constructor(
    private applicationConfig: ApplicationConfigService,
    private hostService: HostsService,
    private backupsService: BackupsService,
    private refcntService: RefCntService,
    private poolService: PoolService,
    private queueTasksService: QueueTasksService,
    private fsckService: FsckService,
  ) {
    super();
  }

  async process(job: Job<RefcntJobData, void, string>): Promise<void> {
    this.#logger.log(`START: Processing job REFCNT ${job.id} : ${job.data.host} - ${job.data.number}: ${job.name}`);
    switch (job.name) {
      case 'add_backup': {
        const informations = this.#prepareAddBackupTask(job);
        await this.queueTasksService.executeTasksFromJob(job, informations);
        break;
      }
      case 'remove_backup': {
        const informations = this.#prepareRemoveBackupTask(job);
        await this.queueTasksService.executeTasksFromJob(job, informations);
        break;
      }
      case 'unused': {
        const informations = this.#prepareCleanUpTask(job);
        await this.queueTasksService.executeTasksFromJob(job, informations);
        break;
      }
      case 'fsck': {
        const informations = await this.#prepareFsck(job);
        await this.queueTasksService.executeTasksFromJob(job, informations);
        break;
      }
      case 'verify_checksum': {
        const informations = await this.#prepareVerifyChecksum(job);
        await this.queueTasksService.executeTasksFromJob(job, informations);
        break;
      }
      default:
        throw new NotFoundException(`Unknown job name ${job.name}`);
    }
    this.#logger.log(`END: Processing job REFCNT ${job.id} : ${job.data.host} - ${job.data.number}: ${job.name}`);
  }

  #prepareAddBackupTask(job: Job<RefcntJobData>) {
    const { host, number, originalDate } = job.data;
    if (!host || number === undefined) {
      throw new BadRequestException(`Host and backup number should be defined`);
    }

    const task = new QueueTasks('GLOBAL', {}).add(new QueueSubTask(RefcntTaskNameEnum.ADD_REFCNT_POOL_TASK));

    return new QueueTasksInformations(task, this.#createGlobalContext(host, number, undefined, originalDate));
  }

  #prepareRemoveBackupTask(job: Job<RefcntJobData>) {
    const { host, number } = job.data;
    if (!host || number === undefined) {
      throw new BadRequestException(`Host and backup number should be defined`);
    }

    const task = new QueueTasks('GLOBAL', {}).add(new QueueSubTask(RefcntTaskNameEnum.REMOVE_REFCNT_POOL_TASK));

    return new QueueTasksInformations(task, this.#createGlobalContext(host, number));
  }

  #prepareCleanUpTask(job: Job<RefcntJobData>) {
    const task = new QueueTasks('GLOBAL', {})
      .add(
        new QueueSubTask(
          RefcntTaskNameEnum.PREPARE_CLEAN_UNUSED_POOL_TASK,
          undefined,
          QueueTaskPriority.PRE_PROCESSING,
        ),
      )
      .add(new QueueSubTask(RefcntTaskNameEnum.CLEAN_UNUSED_POOL_TASK));

    return new QueueTasksInformations(task, this.#createGlobalContext(undefined, undefined, job.data.target));
  }

  async #prepareFsck(job: Job<RefcntJobData>) {
    const { fix, refcnt, unused } = job.data;
    const dryRun = !fix;

    const logger = new JobLogger(this.applicationConfig, job);
    const globalContext = new QueueTaskContext({}, logger);

    const hosts = await this.hostService.getHosts();
    const backups = (
      await Promise.all(
        hosts.map(async (host) =>
          (await this.backupsService.getBackups(host)).map((backup) => ({ host, number: backup.number })),
        ),
      )
    ).flat();

    globalContext.commands.set('prepare', async (_gc, {}) => {
      return new QueueTaskProgression({ progressMax: BigInt(hosts.length + backups.length + 1) });
    });
    globalContext.commands.set('refcnt_backup', async (gc, { host, number }) => {
      if (!host) throw new Error('host is required');
      if (number === undefined) throw new Error('number is required');

      return await this.fsckService.checkBackupIntegrity(logger, host, number, dryRun);
    });
    globalContext.commands.set('refcnt_host', async (gc, { host }) => {
      if (!host) throw new Error('host is required');

      return await this.fsckService.checkHostIntegrity(logger, host, dryRun);
    });
    globalContext.commands.set('refcnt_pool', async () => this.fsckService.checkPoolIntegrity(logger, dryRun));
    globalContext.commands.set('refcnt_unused', () => this.fsckService.processUnused(logger, dryRun));

    const tasks = new QueueTasks('GLOBAL', {}).add(new QueueSubTask('prepare', {}, QueueTaskPriority.INITIALISATION));

    if (refcnt) {
      for (const backup of backups) {
        tasks.add(
          new QueueSubTask('refcnt_backup', { host: backup.host, number: backup.number }, QueueTaskPriority.PROCESSING),
        );
      }
      for (const host of hosts) {
        tasks.add(new QueueSubTask('refcnt_host', { host }, QueueTaskPriority.PROCESSING));
      }
      tasks.add(new QueueSubTask('refcnt_pool', {}, QueueTaskPriority.PROCESSING));
    }
    if (unused) {
      tasks.add(new QueueSubTask('refcnt_unused', {}, QueueTaskPriority.PROCESSING));
    }

    return new QueueTasksInformations(tasks, globalContext);
  }

  async #prepareVerifyChecksum(job: Job<RefcntJobData>) {
    const logger = new JobLogger(this.applicationConfig, job);
    const globalContext = new QueueTaskContext({}, logger);

    globalContext.commands.set('prepare', async () => {
      return new QueueTaskProgression({ progressMax: BigInt(await this.fsckService.countAllChunks()) });
    });
    globalContext.commands.set(RefcntTaskNameEnum.VERIFY_CHECKSUM, () => {
      return this.fsckService.processVerifyChunk(logger);
    });

    const tasks = new QueueTasks('GLOBAL', {}).add(new QueueSubTask('prepare', {}, QueueTaskPriority.INITIALISATION));

    tasks.add(new QueueSubTask(RefcntTaskNameEnum.VERIFY_CHECKSUM, {}, QueueTaskPriority.PROCESSING));

    return new QueueTasksInformations(tasks, globalContext);
  }

  #createGlobalContext(hostname?: string, backupNumber?: number, target?: string, originalDate?: number) {
    const logger = new BackupLogger(this.backupsService, hostname ?? 'refcnt', backupNumber);
    const refcnt = new ReferenceCount(
      (hostname && this.backupsService.getHostDirectory(hostname)) ?? '',
      (hostname && backupNumber?.toString() && this.backupsService.getDestinationDirectory(hostname, backupNumber)) ??
        '',
      this.applicationConfig.poolPath,
    );

    const globalContext = new QueueTaskContext(refcnt, logger);

    globalContext.commands.set(RefcntTaskNameEnum.ADD_REFCNT_POOL_TASK, async () => {
      await this.refcntService.addBackupRefcntTo(
        refcnt.poolPath,
        refcnt.backupPath,
        refcnt.unusedPoolPath,
        originalDate,
      );
    });
    globalContext.commands.set(RefcntTaskNameEnum.REMOVE_REFCNT_POOL_TASK, async () => {
      await this.refcntService.removeBackupRefcntTo(refcnt.poolPath, refcnt.backupPath, refcnt.unusedPoolPath);
    });
    globalContext.commands.set(RefcntTaskNameEnum.PREPARE_CLEAN_UNUSED_POOL_TASK, async () => {
      const numberOfUnusedFiles = await this.poolService.countUnusedFiles();
      return new QueueTaskProgression({
        progressMax: BigInt(numberOfUnusedFiles),
      });
    });
    globalContext.commands.set(RefcntTaskNameEnum.CLEAN_UNUSED_POOL_TASK, () => {
      return this.poolService.removeUnusedFiles(target).pipe(
        scan((acc, val) => {
          return new QueueTaskProgression({
            progressCurrent: acc.progressCurrent + 1n,
            fileCount: acc.fileCount + 1,
            fileSize: acc.fileSize + BigInt(val.size || 0),
            compressedFileSize: acc.compressedFileSize + BigInt(val.compressedSize || 0),
          });
        }, new QueueTaskProgression()),
      );
    });

    return globalContext;
  }
}
