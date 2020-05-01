import { InjectQueue } from '@nestjs/bull';
import { Inject } from '@nestjs/common';
import { Args, Int, Query, Resolver, Subscription } from '@nestjs/graphql';
import { JobStatus, Queue } from 'bull';
import { PubSub } from 'graphql-subscriptions';

import { BackupTask, Job } from '../tasks/tasks.dto';

@Resolver()
export class QueueResolver {
  constructor(
    @InjectQueue('queue') private backupQueue: Queue<BackupTask>,
    @Inject('BACKUP_QUEUE_PUB_SUB') private pubSub: PubSub,
  ) {}

  @Query(() => [Job])
  async queue(@Args('state', { type: () => [String], defaultValue: [] }) states: JobStatus[]) {
    return await this.backupQueue.getJobs(states);
  }

  @Subscription(() => Job)
  jobUpdated() {
    return this.pubSub.asyncIterator('jobUpdated');
  }

  @Subscription(() => Int)
  jobWaiting() {
    return this.pubSub.asyncIterator('jobWaiting');
  }

  @Subscription(() => Job)
  jobFailed() {
    return this.pubSub.asyncIterator('jobFailed');
  }

  @Subscription(() => Job)
  jobRemoved() {
    return this.pubSub.asyncIterator('jobRemoved');
  }
}
