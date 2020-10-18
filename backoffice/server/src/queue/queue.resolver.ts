import { InjectQueue } from '@nestjs/bull';
import { Inject } from '@nestjs/common';
import { Args, Int, Query, Resolver, Subscription } from '@nestjs/graphql';
import Bull, { JobStatus, Queue, Job as BullJob } from 'bull';
import * as cronParser from 'cron-parser';
import { PubSub } from 'graphql-subscriptions';

import { SchedulerConfigService } from '../scheduler/scheduler-config.service';
import { BackupTask, Job } from '../tasks/tasks.dto';
import { QueueStats } from './queue.model';

@Resolver()
export class QueueResolver {
  constructor(
    @InjectQueue('queue') private backupQueue: Queue<BackupTask>,
    @Inject('BACKUP_QUEUE_PUB_SUB') private pubSub: PubSub,
    private scheduler: SchedulerConfigService,
  ) {}

  @Query(() => [Job])
  async queue(
    @Args('state', { type: () => [String], defaultValue: [] }) states: JobStatus[],
  ): Promise<Bull.Job<BackupTask>[]> {
    return await this.backupQueue.getJobs(states);
  }

  @Query(() => QueueStats)
  async queueStats(): Promise<QueueStats> {
    const interval = cronParser.parseExpression((await this.scheduler.getScheduler()).wakeupSchedule);
    return {
      waiting: await this.backupQueue.getWaitingCount(),
      active: await this.backupQueue.getActiveCount(),
      failed: await this.backupQueue.getFailedCount(),
      delayed: await this.backupQueue.getDelayedCount(),
      completed: await this.backupQueue.getCompletedCount(),

      lastExecution: interval.prev().toDate().getTime(),
      nextWakeup: interval.next().toDate().getTime(),
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
