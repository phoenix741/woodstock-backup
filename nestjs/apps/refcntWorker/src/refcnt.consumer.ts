import { Processor, WorkerHost } from '@nestjs/bullmq';
import { BadRequestException, Logger, NotFoundException } from '@nestjs/common';
import { ApplicationConfigService, BackupsService, HostsService, PoolService, RefcntJobData } from '@woodstock/shared';
import {
  QueueSubTask,
  QueueTaskContext,
  QueueTaskPriority,
  QueueTaskProgression,
  QueueTasks,
  QueueTasksInformations,
  QueueTasksService,
} from '@woodstock/shared/tasks';
import { Job } from 'bullmq';
import { map, scan } from 'rxjs';

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
    private poolService: PoolService,
    private queueTasksService: QueueTasksService,
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
        const informations = await this.#prepareVerifyChecksum();
        await this.queueTasksService.executeTasksFromJob(job, informations);
        break;
      }
      default:
        throw new NotFoundException(`Unknown job name ${job.name}`);
    }
    this.#logger.log(`END: Processing job REFCNT ${job.id} : ${job.data.host} - ${job.data.number}: ${job.name}`);
  }

  #prepareAddBackupTask(job: Job<RefcntJobData>) {
    const { host, number } = job.data;
    if (!host || number === undefined) {
      throw new BadRequestException(`Host and backup number should be defined`);
    }

    const task = new QueueTasks('GLOBAL', {}).add(new QueueSubTask(RefcntTaskNameEnum.ADD_REFCNT_POOL_TASK));

    return new QueueTasksInformations(task, this.#createGlobalContext(host, number, undefined));
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
    const { fix, unused } = job.data;
    const dryRun = !fix;

    const globalContext = new QueueTaskContext({});

    globalContext.commands.set('prepare', async (_gc, {}) => {
      const count = (await this.poolService.verifyRefcntMax()) + (await this.poolService.verifyUnusedMax());
      return new QueueTaskProgression({ progressMax: BigInt(count) });
    });
    globalContext.commands.set('refcnt_backup', () => {
      return this.poolService.verifyRefcnt(dryRun).pipe(
        map((val) => {
          return new QueueTaskProgression({
            progressCurrent: BigInt(val.progressCurrent),
          });
        }),
      );
    });
    globalContext.commands.set('refcnt_unused', () => {
      return this.poolService
        .verifyUnused(dryRun)
        .pipe(map((val) => new QueueTaskProgression({ progressCurrent: BigInt(val.progressCurrent) })));
    });

    const tasks = new QueueTasks('GLOBAL', {}).add(new QueueSubTask('prepare', {}, QueueTaskPriority.INITIALISATION));

    tasks.add(new QueueSubTask('refcnt_backup', {}, QueueTaskPriority.PROCESSING));
    if (unused) {
      tasks.add(new QueueSubTask('refcnt_unused', {}, QueueTaskPriority.PROCESSING));
    }

    return new QueueTasksInformations(tasks, globalContext);
  }

  async #prepareVerifyChecksum() {
    const globalContext = new QueueTaskContext({});

    globalContext.commands.set('prepare', async () => {
      return new QueueTaskProgression({ progressMax: BigInt(await this.poolService.countChunk()) });
    });
    globalContext.commands.set(RefcntTaskNameEnum.VERIFY_CHECKSUM, () => {
      return this.poolService
        .verifyChunk()
        .pipe(map((val) => new QueueTaskProgression({ progressCurrent: BigInt(val.totalCount) })));
    });

    const tasks = new QueueTasks('GLOBAL', {}).add(new QueueSubTask('prepare', {}, QueueTaskPriority.INITIALISATION));

    tasks.add(new QueueSubTask(RefcntTaskNameEnum.VERIFY_CHECKSUM, {}, QueueTaskPriority.PROCESSING));

    return new QueueTasksInformations(tasks, globalContext);
  }

  #createGlobalContext(hostname?: string, backupNumber?: number, target?: string) {
    const globalContext = new QueueTaskContext({});

    globalContext.commands.set(RefcntTaskNameEnum.ADD_REFCNT_POOL_TASK, async () => {
      if (hostname === undefined || backupNumber === undefined) {
        throw new Error('Hostname and backup number are required');
      }

      await this.poolService.addRefcntOfPool(hostname, backupNumber);
    });
    globalContext.commands.set(RefcntTaskNameEnum.REMOVE_REFCNT_POOL_TASK, async () => {
      if (hostname === undefined || backupNumber === undefined) {
        throw new Error('Hostname and backup number are required');
      }

      await this.poolService.removeRefcntOfPool(hostname, backupNumber);
    });
    globalContext.commands.set(RefcntTaskNameEnum.PREPARE_CLEAN_UNUSED_POOL_TASK, async () => {
      const numberOfUnusedFiles = await this.poolService.countUnused();
      return new QueueTaskProgression({
        progressMax: BigInt(numberOfUnusedFiles),
      });
    });
    globalContext.commands.set(RefcntTaskNameEnum.CLEAN_UNUSED_POOL_TASK, () => {
      return this.poolService.removeUnused(target).pipe(
        scan((acc, val) => {
          return new QueueTaskProgression({
            progressCurrent: acc.progressCurrent + 1n,
            fileCount: acc.fileCount + 1,
            fileSize: acc.fileSize + BigInt(val.fileSize || 0),
            compressedFileSize: acc.compressedFileSize + BigInt(val.compressedFileSize || 0),
          });
        }, new QueueTaskProgression()),
      );
    });

    return globalContext;
  }
}
