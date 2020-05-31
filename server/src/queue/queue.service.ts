import {
  OnQueueActive,
  OnQueueCleaned,
  OnQueueCompleted,
  OnQueueError,
  OnQueueFailed,
  OnQueueProgress,
  OnQueueRemoved,
  OnQueueStalled,
  OnQueueWaiting,
  Processor,
} from '@nestjs/bull';
import { Inject, Logger } from '@nestjs/common';
import { Job, Queue } from 'bull';
import { PubSub } from 'graphql-subscriptions';

import { BackupTask } from '../tasks/tasks.dto';
import { InjectQueue } from '@nestjs/bull';

@Processor('queue')
export class QueueService {
  private logger = new Logger(QueueService.name);

  constructor(
    @InjectQueue('queue') private backupQueue: Queue<BackupTask>,
    @Inject('BACKUP_QUEUE_PUB_SUB') private pubSub: PubSub,
  ) {}

  @OnQueueError()
  onError(err: Error) {
    this.logger.error(`Error while processing the queue: ${err.message}`, err.stack);
  }

  @OnQueueWaiting()
  onWaiting(jobId: number | string) {
    this.pubSub.publish('jobWaiting', { jobWaiting: jobId });
  }

  @OnQueueCompleted()
  async onCompleted(job: Job<BackupTask>) {
    await this.removeHost(job);
    this.pubSub.publish('jobUpdated', { jobUpdated: job });
  }

  @OnQueueProgress()
  async onProgress(job: Job<BackupTask>) {
    this.pubSub.publish('jobUpdated', { jobUpdated: job });
  }

  @OnQueueStalled()
  async onStalled(job: Job<BackupTask>) {
    this.logger.warn(`Job ${job.id}, for the host ${job.data.host} was stalled.`);
    await this.removeHost(job);
    this.pubSub.publish('jobUpdated', { jobUpdated: job });
  }

  @OnQueueFailed()
  async onFailed(job: Job<BackupTask>, err: Error) {
    this.logger.error(`Error when processing the job ${job.id} with the error ${err.message}`, err.stack);
    await this.removeHost(job);
    this.pubSub.publish('jobUpdated', { jobUpdated: job });
    this.pubSub.publish('jobFailed', { jobFailed: job });
  }

  @OnQueueCleaned()
  onCleaned(jobs: Job<BackupTask>[]) {
    for (const job of jobs) {
      this.pubSub.publish('jobUpdated', { jobUpdated: job });
      this.pubSub.publish('jobRemoved', { jobRemoved: job });
    }
  }

  @OnQueueRemoved()
  onRemoved(job: Job<BackupTask>) {
    this.pubSub.publish('jobUpdated', { jobUpdated: job });
    this.pubSub.publish('jobRemoved', { jobRemoved: job });
  }

  async removeHost(job: Job<BackupTask>) {
    const backups = await this.backupQueue.getJobs([]);
    const backupToRemove = backups.filter(
      j => ['backup', 'stats'].includes(j.name) && j.id !== job.id && j.data.host && j.data.host === job.data.host,
    );
    for (const jobToRemove of backupToRemove) {
      if (!(await jobToRemove.isActive())) {
        await jobToRemove.remove();
      }
    }
  }
}
