import { InjectQueue, QueueEventsHost, QueueEventsListener } from '@nestjs/bullmq';
import { Injectable, Logger } from '@nestjs/common';
import { Job, JobState, Queue } from 'bullmq';
import * as cronParser from 'cron-parser';

import { BackupsService, HostsService, LockService } from '../backups';
import { QueueName } from '../queue';
import { JobBackupData } from './backuping.model';
import { RefcntJobData } from '../models';
import { SchedulerConfigService } from '../config';
import { PingService } from '../commands';

const RUN_JOB_STATE: JobState[] = ['active', 'delayed', 'waiting', 'waiting-children'];

export const LOCK_TIMEOUT = 60_000;

@Injectable()
@QueueEventsListener(QueueName.REFCNT_QUEUE)
export class JobService extends QueueEventsHost {
  private logger = new Logger(JobService.name);

  constructor(
    @InjectQueue(QueueName.BACKUP_QUEUE) private hostsQueue: Queue<JobBackupData>,
    @InjectQueue(QueueName.REFCNT_QUEUE) private refcnQueue: Queue<RefcntJobData>,
    private lockService: LockService,
    private hostsService: HostsService,
    private backupsService: BackupsService,
    private schedulerConfigService: SchedulerConfigService,
    private pingService: PingService,
  ) {
    super();
  }

  async getTimeToNextBackup(hostname: string): Promise<number | undefined> {
    const schedule = await this.hostsService.getSchedule(hostname);
    if (!schedule.activated) {
      this.logger.debug(`Backup is not activated for the host ${hostname}`);
      return undefined;
    }

    // Time since last backup : If backup is activated, and the last backup is old, we crete a new backup
    const timeSinceLastBackup = await this.backupsService.getTimeSinceLastBackup(hostname);
    if (timeSinceLastBackup === undefined) {
      this.logger.debug(`No backup for the host ${hostname}`);
      return 0;
    }
    const lastBackup = await this.backupsService.getLastBackup(hostname);
    if (!lastBackup || !lastBackup.completed) {
      this.logger.debug(`Last backup for the host ${hostname} is not completed`);
      return 0;
    }

    const backupPeriod = schedule.backupPeriod ?? 0;

    this.logger.debug(
      `Last backup for the host ${hostname} have been made at ${
        timeSinceLastBackup / 3600
      } hours past (should be made after ${backupPeriod / 3600} hour)`,
    );

    return Math.max(0, backupPeriod - timeSinceLastBackup);
  }

  async getDateToNextBackup(hostname: string): Promise<Date | undefined> {
    const timeToNextBackupSecs = await this.getTimeToNextBackup(hostname);
    if (timeToNextBackupSecs === undefined) {
      return undefined;
    }

    const timeToNextBackup = timeToNextBackupSecs * 1000;
    const date = new Date(Date.now() + timeToNextBackup);

    let schedulerConfig = await this.schedulerConfigService.getScheduler();
    const interval = cronParser.parseExpression(schedulerConfig.wakeupSchedule!, {
      currentDate: date,
    });

    return interval.next().toDate();
  }

  /**
   * Lock the job during it's execution
   * @param job The job to lock
   * @param routine The code to execute
   * @returns A value to return
   */
  using<T>(job: Job<JobBackupData>, routine: (signal: AbortSignal) => Promise<T>): Promise<T> {
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

    // Time since last backup : If backup is activated, and the last backup is old, we create a new backup
    const timeToNextBackup = await this.getTimeToNextBackup(host);
    this.logger.debug(`Time to next backup for ${host}: ${timeToNextBackup}`);

    if (!force) {
      // TODO: Check period of inactivity vs period of activity
      if (timeToNextBackup === undefined || timeToNextBackup > 0) {
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
    const config = await this.hostsService.getHost(host);
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
    host: string,
    number: number,
    operation: 'add_backup' | 'remove_backup',
  ): Promise<void> {
    this.logger.log(`Launch ${operation} for ${host}`);
    const job = await this.refcnQueue.add(
      operation,
      {
        host,
        number,
      },
      {
        parent: {
          id: jobid,
          queue: jobname,
        },
        removeOnComplete: true,
      },
    );

    await job.waitUntilFinished(this.queueEvents);
    this.logger.log(`${operation} for ${host} finished`);
  }
}
