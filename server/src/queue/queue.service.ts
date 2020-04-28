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
import { Job } from 'bull';
import { PubSub } from 'graphql-subscriptions';

import { BackupTask } from '../tasks/tasks.dto';

@Processor('queue')
export class QueueService {
  private logger = new Logger(QueueService.name);

  constructor(@Inject('BACKUP_QUEUE_PUB_SUB') private pubSub: PubSub) {}

  @OnQueueError()
  onError(err: Error) {
    this.logger.error(`Error while processing the queue: ${err.message}`, err.stack);
  }

  @OnQueueActive()
  onActive(job: Job<BackupTask>) {
    this.pubSub.publish('jobUpdated', { jobUpdated: job });
  }

  @OnQueueWaiting()
  onWaiting(jobId: number | string) {
    this.pubSub.publish('jobWaiting', { jobWaiting: jobId });
  }

  @OnQueueCompleted()
  onCompleted(job: Job<BackupTask>) {
    this.pubSub.publish('jobUpdated', { jobUpdated: job });
  }

  @OnQueueProgress()
  onProgress(job: Job<BackupTask>) {
    this.pubSub.publish('jobUpdated', { jobUpdated: job });
  }

  @OnQueueStalled()
  onStalled(job: Job<BackupTask>) {
    this.logger.warn(`Job ${job.id}, for the host ${job.data.host} was stalled.`);
  }

  @OnQueueFailed()
  onFailed(job: Job<BackupTask>, err: Error) {
    this.logger.error(`Error when processing the job ${job.id} with the error ${err.message}`, err.stack);
    this.pubSub.publish('jobFailed', { jobFailed: job });
  }

  @OnQueueCleaned()
  onCleaned(jobs: Job<BackupTask>[]) {
    for (const job of jobs) {
      this.pubSub.publish('jobRemoved', { jobRemoved: job });
    }
  }

  @OnQueueRemoved()
  onRemoved(job: Job<BackupTask>) {
    this.pubSub.publish('jobRemoved', { jobRemoved: job });
  }
}
