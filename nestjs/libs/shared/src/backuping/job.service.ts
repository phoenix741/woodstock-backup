import { InjectQueue, QueueEventsHost, QueueEventsListener } from '@nestjs/bullmq';
import { Injectable, Logger } from '@nestjs/common';
import { Job, JobState, Queue } from 'bullmq';
import { RedlockAbortSignal } from 'redlock';
import { PingService } from '../commands';
import { BackupsService, HostsService, SchedulerConfigService } from '../config';
import { RefcntJobData } from '../pool';
import { BackupTask } from '../shared';
import { LockService } from '../shared/lock.service';
import { JobBackupData } from './backuping.model';

const RUN_JOB_STATE: JobState[] = ['active', 'delayed', 'waiting', 'waiting-children'];

export const LOCK_TIMEOUT = 5000;

@Injectable()
@QueueEventsListener('refcnt')
export class JobService extends QueueEventsHost {
  private logger = new Logger(JobService.name);

  constructor(
    @InjectQueue('queue') private hostsQueue: Queue<JobBackupData>,
    @InjectQueue('refcnt') private refcnQueue: Queue<RefcntJobData>,
    private lockService: LockService,
    private hostsService: HostsService,
    private backupsService: BackupsService,
    private schedulerConfigService: SchedulerConfigService,
    private pingService: PingService,
  ) {
    super();
  }

  /**
   * Lock the job during it's execution
   * @param job The job to lock
   * @param routine The code to execute
   * @returns A value to return
   */
  using<T>(job: Job<JobBackupData>, routine: (signal: RedlockAbortSignal) => Promise<T>): Promise<T> {
    return this.lockService.using([job.data.host], LOCK_TIMEOUT, routine);
  }

  /**
   * Check if the job is locked
   * @param job The job to check
   * @returns A boolean to indicate if the value is locked
   */
  async isLocked(job: Job<JobBackupData> | string): Promise<boolean> {
    if (typeof job === 'string') {
      return await this.lockService.isLocked([job]);
    }
    return await this.lockService.isLocked([job.data.host]);
  }

  /**
   * This service is used to check if a backup should be launch.
   * A backup should be launch if:
   *
   * - the job isn't active
   * - the job isn't locked
   * - the time since last backup is greater than the backup interval (and the backup isn't complete)
   *
   * @param host The host to check
   * @returns true if the backup should be launch, false otherwise
   */
  async shouldBackupHost(host: string, jobId?: string, force = false): Promise<boolean> {
    // Have already backup
    const runningJob = await this.hostsQueue.getJobs(RUN_JOB_STATE);
    const runningJobForHost = runningJob.find((b) => b.data.host === host && b.id !== jobId);
    if (runningJobForHost) {
      this.logger.debug(
        `A job is already running for ${host}: ${runningJobForHost.id}/${
          runningJobForHost.name
        }/${await runningJobForHost.getState()}`,
      );
      return false;
    }

    // Lock
    const isLocked = await this.isLocked(host);
    if (isLocked) {
      this.logger.warn(`A job is already running for ${host}, but is not in the queue`);
      return false;
    }

    // Time since last backup : If backup is activated, and the last backup is old, we crete a new backup
    const config = await this.hostsService.getHostConfiguration(host);
    const schedulerConfig = await this.schedulerConfigService.getScheduler();
    const schedule = Object.assign({}, schedulerConfig.defaultSchedule, config.schedule);

    const lastBackup = await this.backupsService.getLastBackup(host);
    const timeSinceLastBackup = (new Date().getTime() - (lastBackup?.startDate || 0)) / 1000;
    const backupPeriod = schedule.backupPeriod || 0;
    this.logger.debug(
      `Last backup for the host ${host} have been made at ${
        timeSinceLastBackup / 3600
      } hours past (should be made after ${backupPeriod / 3600} hour)`,
    );

    if (!force) {
      // TODO: Check period of inactivity vs period of activity
      if (!(schedule.activated && (!lastBackup?.complete || timeSinceLastBackup > backupPeriod))) {
        return false;
      }
    }

    return true;
  }

  /**
   * Is the host available on the network
   * @param host The host to check
   * @returns true if the host is available, false otherwise
   */
  async hostAvailable(host: string): Promise<boolean> {
    const config = await this.hostsService.getHostConfiguration(host);
    const isHostAvailable = await this.pingService.pingFromConfig(host, config);
    if (!isHostAvailable) {
      this.logger.debug(`Host ${host} not available on network`);
      return false;
    }

    return true;
  }

  /**
   * Get the last backup for a host.
   * @param host The host to check
   * @returns The last backup number and the previous backup number
   */
  async getNextBackup(host: string): Promise<Pick<JobBackupData, 'number' | 'previousNumber'>> {
    const lastBackup = await this.backupsService.getLastBackup(host);
    if (lastBackup) {
      return { number: lastBackup.number + 1, previousNumber: lastBackup.number };
    } else {
      return { number: 0, previousNumber: undefined };
    }
  }

  /**
   * Launch on refcnt queue the children to delete or create a backup.
   */
  async launchRefcntJob(
    jobid: string,
    jobname: string,
    hostname: string,
    backupNumber: number,
    operation: 'add_backup' | 'remove_backup',
  ): Promise<void> {
    this.logger.log(`Launch ${operation} for ${hostname}`);
    const job = await this.refcnQueue.add(
      operation,
      {
        hostname,
        backupNumber,
      },
      {
        parent: {
          id: jobid,
          queue: jobname,
        },
      },
    );

    await job.waitUntilFinished(this.queueEvents);
    this.logger.log(`${operation} for ${hostname} finished`);
  }
}
