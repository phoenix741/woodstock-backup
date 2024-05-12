import { InjectQueue, OnQueueEvent, QueueEventsHost, QueueEventsListener } from '@nestjs/bullmq';
import { Inject, Logger } from '@nestjs/common';
import { JobBackupData, QueueName } from '@woodstock/shared';
import { Queue } from 'bullmq';
import { PubSub } from 'graphql-subscriptions';
import { QueueUtils } from './queue.utils';

@QueueEventsListener(QueueName.BACKUP_QUEUE)
export class QueueService extends QueueEventsHost {
  private logger = new Logger(QueueService.name);

  constructor(
    @InjectQueue(QueueName.BACKUP_QUEUE) private backupQueue: Queue<JobBackupData>,
    @Inject('BACKUP_QUEUE_PUB_SUB') private pubSub: PubSub,
    private queueUtils: QueueUtils,
  ) {
    super();
  }

  @OnQueueEvent('active')
  async onActive({ jobId }: { jobId: string }): Promise<void> {
    try {
      const job = await this.backupQueue.getJob(jobId);
      if (job) {
        this.logger.log(`Job ${job.id} for the host ${job.data.host} was active.`);
        this.pubSub.publish('jobUpdated', { jobUpdated: await this.queueUtils.getJob(job) });
      }
    } catch (err) {
      this.logger.error(`Error while processing the queue: ${err.message}`, err.stack);
      console.error(err);
    }
  }

  @OnQueueEvent('added')
  async onAdded({ jobId }: { jobId: string }): Promise<void> {
    try {
      const job = await this.backupQueue.getJob(jobId);
      if (job) {
        this.logger.log(`Job ${job.id} for the host ${job.data.host} was added.`);
        this.pubSub.publish('jobUpdated', { jobUpdated: await this.queueUtils.getJob(job) });
      }
    } catch (err) {
      this.logger.error(`Error while processing the queue: ${err.message}`, err.stack);
      console.error(err);
    }
  }

  @OnQueueEvent('delayed')
  async onDelayed({ jobId }: { jobId: string }): Promise<void> {
    try {
      const job = await this.backupQueue.getJob(jobId);
      if (job) {
        this.logger.log(`Job ${job.id} for the host ${job.data.host} was delayed.`);
        this.pubSub.publish('jobUpdated', { jobUpdated: await this.queueUtils.getJob(job) });
      }
    } catch (err) {
      this.logger.error(`Error while processing the queue: ${err.message}`, err.stack);
      console.error(err);
    }
  }

  @OnQueueEvent('error')
  onError(err: Error): void {
    this.logger.error(`Error while processing the queue: ${err.message}`, err.stack);
  }

  @OnQueueEvent('waiting')
  onWaiting({ jobId }: { jobId: string }): void {
    this.logger.log(`Job ${jobId} is waiting`);
    this.pubSub.publish('jobWaiting', { jobWaiting: jobId });
  }

  @OnQueueEvent('completed')
  async onCompleted({ jobId }: { jobId: string }): Promise<void> {
    try {
      const job = await this.backupQueue.getJob(jobId);
      if (job) {
        this.logger.log(`Job ${job.id} for the host ${job.data.host} was completed.`);

        await this.#cleanupJobs(job.name, job.data.host, [job.id ?? '']);

        this.pubSub.publish('jobUpdated', { jobUpdated: await this.queueUtils.getJob(job) });
      }
    } catch (err) {
      this.logger.error(`Error while processing the queue: ${err.message}`, err.stack);
      console.error(err);
    }
  }

  @OnQueueEvent('progress')
  async onProgress({ jobId }: { jobId: string }): Promise<void> {
    try {
      const job = await this.backupQueue.getJob(jobId);
      if (job) {
        this.logger.log(`Job ${job.id} for the host ${job.data.host} is in progress.`);
        this.pubSub.publish('jobUpdated', { jobUpdated: await this.queueUtils.getJob(job) });
      }
    } catch (err) {
      this.logger.error(`Error while processing the queue: ${err.message}`, err.stack);
      console.error(err);
    }
  }

  @OnQueueEvent('stalled')
  async onStalled({ jobId }: { jobId: string }): Promise<void> {
    try {
      const job = await this.backupQueue.getJob(jobId);
      if (job) {
        this.logger.warn(`Job ${job.id}, for the host ${job.data.host} was stalled.`);
        this.pubSub.publish('jobUpdated', { jobUpdated: await this.queueUtils.getJob(job) });
      }
    } catch (err) {
      this.logger.error(`Error while processing the queue: ${err.message}`, err.stack);
      console.error(err);
    }
  }

  @OnQueueEvent('failed')
  async onFailed({ jobId, failedReason }: { jobId: string; failedReason: string }): Promise<void> {
    try {
      const job = await this.backupQueue.getJob(jobId);
      if (job) {
        this.logger.error(`Error when processing the job ${job.id} with the error ${failedReason}`);

        await this.#cleanupJobs(job.name, job.data.host, [job.id ?? '']);

        this.pubSub.publish('jobUpdated', { jobUpdated: await this.queueUtils.getJob(job) });
        this.pubSub.publish('jobFailed', { jobFailed: await this.queueUtils.getJob(job) });
      }
    } catch (err) {
      this.logger.error(`Error while processing the queue: ${err.message}`, err.stack);
      console.error(err);
    }
  }

  @OnQueueEvent('removed')
  async onRemoved({ jobId }: { jobId: string }): Promise<void> {
    try {
      const job = await this.backupQueue.getJob(jobId);
      if (job) {
        this.logger.log(`Job ${job.id} was removed from the queue.`);
        this.pubSub.publish('jobUpdated', { jobUpdated: await this.queueUtils.getJob(job) });
        this.pubSub.publish('jobRemoved', { jobRemoved: await this.queueUtils.getJob(job) });
      }
    } catch (err) {
      this.logger.error(`Error while processing the queue: ${err.message}`, err.stack);
      console.error(err);
    }
  }

  /**
   * Remove all completed and failed jobs for a given hostname from backupQueue
   * @param hostname hostname to remove
   */
  async #cleanupJobs(jobName: string, hostname: string, jobToKeep: string[]) {
    const jobs = await this.backupQueue.getJobs(['completed', 'failed']);
    const jobsToRemove = jobs
      .filter((job) => !!job)
      .filter((job) => job.name === jobName && job.data.host === hostname)
      .filter((job) => !jobToKeep.includes(job.id || ''));
    await Promise.all(jobsToRemove.map((job) => job.remove()));
  }
}
