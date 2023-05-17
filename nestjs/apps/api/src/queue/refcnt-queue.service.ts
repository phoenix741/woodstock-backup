import { InjectQueue, OnQueueEvent, QueueEventsHost, QueueEventsListener } from '@nestjs/bullmq';
import { Inject, Logger } from '@nestjs/common';
import { QueueName, RefcntJobData } from '@woodstock/server';
import { Queue } from 'bullmq';
import { PubSub } from 'graphql-subscriptions';
import { QueueUtils } from './queue.utils';

@QueueEventsListener(QueueName.REFCNT_QUEUE)
export class RefcntQueueService extends QueueEventsHost {
  private logger = new Logger(RefcntQueueService.name);

  constructor(
    @InjectQueue(QueueName.REFCNT_QUEUE) private refcntQueue: Queue<RefcntJobData>,
    @Inject('BACKUP_QUEUE_PUB_SUB') private pubSub: PubSub,
    private queueUtils: QueueUtils,
  ) {
    super();
  }

  @OnQueueEvent('active')
  async onActive({ jobId }: { jobId: string }): Promise<void> {
    const job = await this.refcntQueue.getJob(jobId);
    if (job) {
      this.logger.log(`Refcnt job ${job.id} for the ${job.data.host ? `host ${job.data.host}` : 'pool'} was active.`);
      this.pubSub.publish('jobUpdated', { jobUpdated: await this.queueUtils.getJob(job) });
    }
  }

  @OnQueueEvent('error')
  onError(err: Error): void {
    this.logger.error(`Error while processing the refcnt queue: ${err.message}`, err.stack);
  }

  @OnQueueEvent('completed')
  async onCompleted({ jobId }: { jobId: string }): Promise<void> {
    const job = await this.refcntQueue.getJob(jobId);
    if (job) {
      this.logger.log(
        `Refcnt job ${job.id} for the ${job.data.host ? `host ${job.data.host}` : 'pool'} was completed.`,
      );

      this.pubSub.publish('jobUpdated', { jobUpdated: await this.queueUtils.getJob(job) });
    }
  }

  @OnQueueEvent('progress')
  async onProgress({ jobId }: { jobId: string }): Promise<void> {
    const job = await this.refcntQueue.getJob(jobId);
    if (job) {
      this.logger.log(
        `Refcnt job ${job.id} for the ${job.data.host ? `host ${job.data.host}` : 'pool'} is in progress.`,
      );
      this.pubSub.publish('jobUpdated', { jobUpdated: await this.queueUtils.getJob(job) });
    }
  }

  @OnQueueEvent('failed')
  async onFailed({ jobId, failedReason }: { jobId: string; failedReason: string }): Promise<void> {
    const job = await this.refcntQueue.getJob(jobId);
    if (job) {
      this.logger.error(`Error when processing the refcnt job ${job.id} with the error ${failedReason}`);

      this.pubSub.publish('jobUpdated', { jobUpdated: await this.queueUtils.getJob(job) });
      this.pubSub.publish('jobFailed', { jobFailed: await this.queueUtils.getJob(job) });
    }
  }

  @OnQueueEvent('removed')
  async onRemoved({ jobId }: { jobId: string }): Promise<void> {
    const job = await this.refcntQueue.getJob(jobId);
    if (job) {
      this.logger.log(`Refcnt job ${job.id} was removed from the queue.`);
      this.pubSub.publish('jobUpdated', { jobUpdated: await this.queueUtils.getJob(job) });
      this.pubSub.publish('jobRemoved', { jobRemoved: await this.queueUtils.getJob(job) });
    }
  }
}
