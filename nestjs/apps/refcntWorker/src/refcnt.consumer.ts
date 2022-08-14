import { Processor, WorkerHost } from '@nestjs/bullmq';
import { InternalServerErrorException, Logger, NotFoundException } from '@nestjs/common';
import {
  ApplicationConfigService,
  BackupsService,
  PoolService,
  RefcntJobData,
  RefCntService,
  ReferenceCount,
} from '@woodstock/shared';
import { Job } from 'bullmq';
import { scan } from 'rxjs';

@Processor('refcnt', { concurrency: 1 })
export class RefcntConsumer extends WorkerHost {
  #logger = new Logger(RefcntConsumer.name);

  constructor(
    private applicationConfig: ApplicationConfigService,
    private backupsService: BackupsService,
    private refcntService: RefCntService,
    private poolService: PoolService,
  ) {
    super();
  }

  async process(job: Job<RefcntJobData, void, string>): Promise<void> {
    this.#logger.log(
      `START: Processing job REFCNT ${job.id} : ${job.data.hostname} - ${job.data.backupNumber}: ${job.name}`,
    );
    switch (job.name) {
      case 'add_backup':
        if (!job.data.hostname) {
          throw new InternalServerErrorException('Missing hostname in job data');
        }
        if (job.data.backupNumber === undefined || job.data.backupNumber === null) {
          throw new InternalServerErrorException('Missing backupNumber in job data');
        }

        await this.#processAddBackup(job.data.hostname, job.data.backupNumber);
        break;
      case 'remove_backup':
        if (!job.data.hostname) {
          throw new InternalServerErrorException('Missing hostname in job data');
        }
        if (job.data.backupNumber === undefined || job.data.backupNumber === null) {
          throw new InternalServerErrorException('Missing backupNumber in job data');
        }

        await this.#processRemove(job.data.hostname, job.data.backupNumber);
        break;
      case 'unused':
        await this.#processUnused(job);
        break;
      default:
        throw new NotFoundException(`Unknown job name ${job.name}`);
    }
    this.#logger.log(
      `END: Processing job REFCNT ${job.id} : ${job.data.hostname} - ${job.data.backupNumber}: ${job.name}`,
    );
  }

  async #processAddBackup(hostname: string, n: number): Promise<void> {
    this.#logger.log(`Adding backup ${n} of host ${hostname} to the reference count of the pool`);
    try {
      const refcnt = new ReferenceCount(
        this.backupsService.getHostDirectory(hostname),
        this.backupsService.getDestinationDirectory(hostname, n),
        this.applicationConfig.poolPath,
      );

      await this.refcntService.addBackupRefcntTo(refcnt.poolPath, refcnt.backupPath, refcnt.unusedPoolPath);
    } finally {
      this.#logger.log(`[END] - Added backup ${n} of host ${hostname} to the reference count of the pool`);
    }
  }

  async #processRemove(hostname: string, n: number): Promise<void> {
    this.#logger.log(`Removing backup ${n} of host ${hostname} from the reference count of the pool`);
    try {
      const refcnt = new ReferenceCount(
        this.backupsService.getHostDirectory(hostname),
        this.backupsService.getDestinationDirectory(hostname, n),
        this.applicationConfig.poolPath,
      );

      await this.refcntService.removeBackupRefcntTo(refcnt.poolPath, refcnt.backupPath, refcnt.unusedPoolPath);
    } finally {
      this.#logger.log(`[END] - Removed backup ${n} of host ${hostname} from the reference count of the pool`);
    }
  }

  async #processUnused(job: Job<RefcntJobData>): Promise<void> {
    this.poolService
      .removeUnusedFiles()
      .pipe(
        scan(
          (acc, val) => {
            return {
              count: acc.count + 1,
              size: acc.size + BigInt(val.size || 0),
              compressedSize: acc.compressedSize + BigInt(val.compressedSize || 0),
            };
          },
          {
            count: 0,
            size: 0n,
            compressedSize: 0n,
          },
        ),
      )
      .subscribe({
        next: (count) => {
          job.updateProgress(count);
        },
      });
  }
}
