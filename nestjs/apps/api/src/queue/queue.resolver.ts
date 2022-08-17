import { InjectQueue } from '@nestjs/bullmq';
import { Inject } from '@nestjs/common';
import { Args, Int, Query, Resolver, Subscription } from '@nestjs/graphql';
import { BackupTask, Job, SchedulerConfigService } from '@woodstock/shared';
import Bull, { Job as BullJob, JobState, Queue } from 'bullmq';
import * as cronParser from 'cron-parser';
import { PubSub } from 'graphql-subscriptions';
import { QueueStats } from './queue.dto.js';

@Resolver()
export class QueueResolver {
  constructor(
    @InjectQueue('queue') private backupQueue: Queue<BackupTask>,
    @Inject('BACKUP_QUEUE_PUB_SUB') private pubSub: PubSub,
    private scheduler: SchedulerConfigService,
  ) {}

  @Query(() => [Job])
  async queue(
    @Args('state', { type: () => [String], defaultValue: [] }) states: JobState[],
  ): Promise<Bull.Job<BackupTask>[]> {
    return await this.backupQueue.getJobs(states);
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
      waiting: await this.backupQueue.getWaitingCount(),
      waitingChildren: await this.backupQueue.getWaitingChildrenCount(),
      active: await this.backupQueue.getActiveCount(),
      failed: await this.backupQueue.getFailedCount(),
      delayed: await this.backupQueue.getDelayedCount(),
      completed: await this.backupQueue.getCompletedCount(),

      lastExecution,
      nextWakeup,
    };
  }

  @Subscription(() => Job)
  jobUpdated(): AsyncIterator<{ jobUpdated: BullJob<BackupTask> }> {
    return this.pubSub.asyncIterator('jobUpdated');
  }

  @Subscription(() => Int)
  jobWaiting(): AsyncIterator<{ jobWaiting: number }> {
    return this.pubSub.asyncIterator('jobWaiting');
  }

  @Subscription(() => Job)
  jobFailed(): AsyncIterator<{ jobUpdated: BullJob<BackupTask> }> {
    return this.pubSub.asyncIterator('jobFailed');
  }

  @Subscription(() => Job)
  jobRemoved(): AsyncIterator<{ jobUpdated: BullJob<BackupTask> }> {
    return this.pubSub.asyncIterator('jobRemoved');
  }
}
