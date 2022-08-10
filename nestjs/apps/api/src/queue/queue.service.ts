import { InjectQueue, OnQueueEvent, QueueEventsHost, QueueEventsListener } from '@nestjs/bullmq';
import { Inject, Logger } from '@nestjs/common';
import { BackupTask } from '@woodstock/shared';
import { Queue } from 'bullmq';
import { PubSub } from 'graphql-subscriptions';

@QueueEventsListener('queue')
export class QueueService extends QueueEventsHost {
  private logger = new Logger(QueueService.name);

  constructor(
    @InjectQueue('queue') private backupQueue: Queue<BackupTask>,

    @Inject('BACKUP_QUEUE_PUB_SUB') private pubSub: PubSub,
  ) {
    super();
  }

  @OnQueueEvent('active')
  async onActive({ jobId }: { jobId: string }): Promise<void> {
    const job = await this.backupQueue.getJob(jobId);
    if (job) {
      this.logger.log(`Job ${job.id} for the host ${job.data.host} was active.`);
      this.pubSub.publish('jobUpdated', { jobUpdated: job });
    }
  }

  @OnQueueEvent('added')
  async onAdded({ jobId }: { jobId: string }): Promise<void> {
    const job = await this.backupQueue.getJob(jobId);
    if (job) {
      this.logger.log(`Job ${job.id} for the host ${job.data.host} was added.`);
      this.pubSub.publish('jobUpdated', { jobUpdated: job });
    }
  }

  @OnQueueEvent('delayed')
  async onDelayed({ jobId }: { jobId: string }): Promise<void> {
    const job = await this.backupQueue.getJob(jobId);
    if (job) {
      this.logger.log(`Job ${job.id} for the host ${job.data.host} was delayed.`);
      this.pubSub.publish('jobUpdated', { jobUpdated: job });
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
    const job = await this.backupQueue.getJob(jobId);
    if (job) {
      this.logger.log(`Job ${job.id} for the host ${job.data.host} was completed.`);
      this.pubSub.publish('jobUpdated', { jobUpdated: job });
    }
  }

  @OnQueueEvent('progress')
  async onProgress({ jobId }: { jobId: string }): Promise<void> {
    const job = await this.backupQueue.getJob(jobId);
    if (job) {
      this.logger.log(`Job ${job.id} for the host ${job.data.host} is in progress.`);
      this.pubSub.publish('jobUpdated', { jobUpdated: job });
    }
  }

  @OnQueueEvent('stalled')
  async onStalled({ jobId }: { jobId: string }): Promise<void> {
    const job = await this.backupQueue.getJob(jobId);
    if (job) {
      this.logger.warn(`Job ${job.id}, for the host ${job.data.host} was stalled.`);
      this.pubSub.publish('jobUpdated', { jobUpdated: job });
    }
  }

  @OnQueueEvent('failed')
  async onFailed({ jobId, failedReason }: { jobId: string; failedReason: string }): Promise<void> {
    const job = await this.backupQueue.getJob(jobId);
    if (job) {
      this.logger.error(`Error when processing the job ${job.id} with the error ${failedReason}`);

      this.pubSub.publish('jobUpdated', { jobUpdated: job });
      this.pubSub.publish('jobFailed', { jobFailed: job });
    }
  }

  @OnQueueEvent('removed')
  async onRemoved({ jobId }: { jobId: string }): Promise<void> {
    const job = await this.backupQueue.getJob(jobId);
    if (job) {
      this.logger.log(`Job ${job.id} was removed from the queue.`);
      this.pubSub.publish('jobUpdated', { jobUpdated: job });
      this.pubSub.publish('jobRemoved', { jobRemoved: job });
    }
  }
}
