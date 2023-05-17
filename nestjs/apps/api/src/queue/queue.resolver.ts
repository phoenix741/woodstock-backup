import { InjectQueue } from '@nestjs/bullmq';
import { Inject } from '@nestjs/common';
import { Args, Int, Query, Resolver, Subscription } from '@nestjs/graphql';
import { SchedulerConfigService } from '@woodstock/core';
import { Job, JobBackupData, QueueListInput, QueueName, RefcntJobData } from '@woodstock/server';
import { Queue } from 'bullmq';
import * as cronParser from 'cron-parser';
import { PubSub } from 'graphql-subscriptions';
import { QueueStats } from './queue.dto.js';
import { QueueUtils } from './queue.utils.js';

@Resolver()
export class QueueResolver {
  constructor(
    @InjectQueue(QueueName.BACKUP_QUEUE) private backupQueue: Queue<JobBackupData>,
    @InjectQueue(QueueName.REFCNT_QUEUE) private refcntQueue: Queue<RefcntJobData>,
    @Inject('BACKUP_QUEUE_PUB_SUB') private pubSub: PubSub,
    private scheduler: SchedulerConfigService,
    private queueUtils: QueueUtils,
  ) {}

  @Query(() => [Job])
  async queue(@Args('input', { type: () => QueueListInput }) input: QueueListInput): Promise<Job[]> {
    const jobs = [...(await this.backupQueue.getJobs(input.states)), ...(await this.refcntQueue.getJobs(input.states))];

    return Promise.all(
      jobs
        .filter((job) => !input.queueName || job.queueName === input.queueName)
        .filter((job) => !input.operationName || job.name === input.operationName)
        .sort((a, b) => b.timestamp - a.timestamp)
        .map(async (job) => await this.queueUtils.getJob(job)),
    );
  }

  @Query(() => QueueStats)
  async queueStats(): Promise<QueueStats> {
    const scheduler = await this.scheduler.getScheduler();
    let lastExecution: number | undefined;
    let nextWakeup: number | undefined;
    if (scheduler.wakeupSchedule) {
      const interval = cronParser.parseExpression(scheduler.wakeupSchedule);
      lastExecution = interval.prev().toDate().getTime();
      nextWakeup = interval.next().toDate().getTime();
    }

    return {
      waiting: (await this.backupQueue.getWaitingCount()) + (await this.refcntQueue.getWaitingCount()),
      waitingChildren:
        (await this.backupQueue.getWaitingChildrenCount()) + (await this.refcntQueue.getWaitingChildrenCount()),
      active: (await this.backupQueue.getActiveCount()) + (await this.refcntQueue.getActiveCount()),
      failed: (await this.backupQueue.getFailedCount()) + (await this.refcntQueue.getFailedCount()),
      delayed: (await this.backupQueue.getDelayedCount()) + (await this.refcntQueue.getDelayedCount()),
      completed: (await this.backupQueue.getCompletedCount()) + (await this.refcntQueue.getCompletedCount()),

      lastExecution,
      nextWakeup,
    };
  }

  @Subscription(() => Job)
  jobUpdated(): AsyncIterator<{ jobUpdated: Job }> {
    return this.pubSub.asyncIterator('jobUpdated');
  }

  @Subscription(() => Int)
  jobWaiting(): AsyncIterator<{ jobWaiting: number }> {
    return this.pubSub.asyncIterator('jobWaiting');
  }

  @Subscription(() => Job)
  jobFailed(): AsyncIterator<{ jobUpdated: Job }> {
    return this.pubSub.asyncIterator('jobFailed');
  }

  @Subscription(() => Job)
  jobRemoved(): AsyncIterator<{ jobUpdated: Job }> {
    return this.pubSub.asyncIterator('jobRemoved');
  }
}
