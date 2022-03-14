import {
  InjectQueue,
  OnGlobalQueueCleaned,
  OnGlobalQueueCompleted,
  OnGlobalQueueError,
  OnGlobalQueueFailed,
  OnGlobalQueueProgress,
  OnGlobalQueueRemoved,
  OnGlobalQueueStalled,
  OnGlobalQueueWaiting,
  Processor,
} from '@nestjs/bull';
import { Inject, Logger } from '@nestjs/common';
import { BackupTask } from '@woodstock/backoffice-shared';
import { Job, JobId, Queue } from 'bull';
import { PubSub } from 'graphql-subscriptions';

@Processor('queue')
export class QueueService {
  private logger = new Logger(QueueService.name);

  constructor(
    @InjectQueue('queue') private backupQueue: Queue<BackupTask>,
    @Inject('BACKUP_QUEUE_PUB_SUB') private pubSub: PubSub,
  ) {}

  @OnGlobalQueueError()
  onError(err: Error): void {
    this.logger.error(`Error while processing the queue: ${err.message}`, err.stack);
  }

  @OnGlobalQueueWaiting()
  onWaiting(jobId: number | string): void {
    this.logger.log(`Job ${jobId} is waiting`);
    this.pubSub.publish('jobWaiting', { jobWaiting: jobId });
  }

  @OnGlobalQueueCompleted()
  async onCompleted(jobId: JobId): Promise<void> {
    const job = await this.backupQueue.getJob(jobId);
    this.logger.log(`Job ${job.id} for the host ${job.data.host} was completed.`);
    await this.removeHost(job);
    this.pubSub.publish('jobUpdated', { jobUpdated: job });
  }

  @OnGlobalQueueProgress()
  async onProgress(jobId: JobId): Promise<void> {
    const job = await this.backupQueue.getJob(jobId);
    this.logger.log(`Job ${job.id} for the host ${job.data.host} is in progress.`);
    this.pubSub.publish('jobUpdated', { jobUpdated: job });
  }

  @OnGlobalQueueStalled()
  async onStalled(jobId: JobId): Promise<void> {
    const job = await this.backupQueue.getJob(jobId);
    this.logger.warn(`Job ${job.id}, for the host ${job.data.host} was stalled.`);
    await this.removeHost(job);
    this.pubSub.publish('jobUpdated', { jobUpdated: job });
  }

  @OnGlobalQueueFailed()
  async onFailed(jobId: JobId, err: Error): Promise<void> {
    const job = await this.backupQueue.getJob(jobId);
    this.logger.error(`Error when processing the job ${job.id} with the error ${err.message}`, err.stack);
    await this.removeHost(job);
    this.pubSub.publish('jobUpdated', { jobUpdated: job });
    this.pubSub.publish('jobFailed', { jobFailed: job });
  }

  @OnGlobalQueueCleaned()
  async onCleaned(jobIds: JobId[]): Promise<void> {
    this.logger.log(`${jobIds.length} jobs were removed from the queue.`);
    for (const jobId of jobIds) {
      const job = await this.backupQueue.getJob(jobId);

      this.pubSub.publish('jobUpdated', { jobUpdated: job });
      this.pubSub.publish('jobRemoved', { jobRemoved: job });
    }
  }

  @OnGlobalQueueRemoved()
  async onRemoved(jobId: JobId): Promise<void> {
    const job = await this.backupQueue.getJob(jobId);
    this.logger.log(`Job ${job.id} was removed from the queue.`);
    this.pubSub.publish('jobUpdated', { jobUpdated: job });
    this.pubSub.publish('jobRemoved', { jobRemoved: job });
  }

  async removeHost(job: Job<BackupTask>): Promise<void> {
    this.logger.log(`Removing host ${job.data.host} from the queue.`);

    const backups = await this.backupQueue.getJobs([]);
    const backupToRemove = backups.filter(
      (j) =>
        j && ['backup', 'stats'].includes(j.name) && j.id !== job.id && j.data.host && j.data.host === job.data.host,
    );

    for (const jobToRemove of backupToRemove) {
      if (!(await jobToRemove.isActive())) {
        await jobToRemove.remove();
      }
    }
  }
}
