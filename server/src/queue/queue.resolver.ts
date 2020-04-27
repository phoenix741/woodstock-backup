import { InjectQueue } from '@nestjs/bull';
import { Resolver, Query, ResolveField, Subscription, Int } from '@nestjs/graphql';
import { Queue, Job as BullJob } from 'bull';

import { BackupTask, Job, BackupQueue } from '../tasks/tasks.dto';
import { Inject } from '@nestjs/common';

import { PubSub } from 'graphql-subscriptions';

@Resolver(() => BackupQueue)
export class QueueResolver {
  constructor(
    @InjectQueue('queue') private backupQueue: Queue<BackupTask>,
    @Inject('BACKUP_QUEUE_PUB_SUB') private pubSub: PubSub,
  ) {}

  @Query(() => BackupQueue)
  queue() {
    return {};
  }

  @ResolveField(() => [Job])
  async waiting() {
    return (await this.backupQueue.getJobs(['waiting'])).map(job => job.toJSON());
  }

  @ResolveField(() => [Job])
  async active() {
    return (await this.backupQueue.getJobs(['active'])).map(job => job.toJSON());
  }

  @ResolveField(() => [Job])
  async failed() {
    return (await this.backupQueue.getJobs(['failed'])).map(job => job.toJSON());
  }

  @ResolveField(() => [Job])
  async delayed() {
    return (await this.backupQueue.getJobs(['delayed'])).map(job => job.toJSON());
  }

  @ResolveField(() => [Job])
  async completed() {
    return (await this.backupQueue.getJobs(['completed'])).map(job => job.toJSON());
  }

  @Subscription(() => Job, {
    resolve: (value: { jobUpdated: BullJob }) => value.jobUpdated.toJSON(),
  })
  jobUpdated() {
    return this.pubSub.asyncIterator('jobUpdated');
  }

  @Subscription(() => Int)
  jobWaiting() {
    return this.pubSub.asyncIterator('jobWaiting');
  }

  @Subscription(() => Job, {
    resolve: (value: { jobFailed: BullJob }) => value.jobFailed.toJSON(),
  })
  jobFailed() {
    return this.pubSub.asyncIterator('jobFailed');
  }

  @Subscription(() => Job, {
    resolve: (value: { jobRemoved: BullJob }) => value.jobRemoved.toJSON(),
  })
  jobRemoved() {
    return this.pubSub.asyncIterator('jobRemoved');
  }
}
