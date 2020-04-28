import { InjectQueue } from '@nestjs/bull';
import { Inject, Logger } from '@nestjs/common';
import { Int, Query, ResolveField, Resolver, Subscription } from '@nestjs/graphql';
import { Queue } from 'bull';
import { PubSub } from 'graphql-subscriptions';

import { BackupQueue, BackupTask, Job } from '../tasks/tasks.dto';

@Resolver(() => BackupQueue)
export class QueueResolver {
  private logger = new Logger(QueueResolver.name);

  constructor(
    @InjectQueue('queue') private backupQueue: Queue<BackupTask>,
    @Inject('BACKUP_QUEUE_PUB_SUB') private pubSub: PubSub,
  ) {}

  @Query(() => BackupQueue)
  queue() {
    return {};
  }

  @ResolveField(() => [Job])
  async all() {
    return await this.backupQueue.getJobs([]);
  }

  @ResolveField(() => [Job])
  async waiting() {
    return await this.backupQueue.getJobs(['waiting']);
  }

  @ResolveField(() => [Job])
  async active() {
    return await this.backupQueue.getJobs(['active']);
  }

  @ResolveField(() => [Job])
  async failed() {
    return await this.backupQueue.getJobs(['failed']);
  }

  @ResolveField(() => [Job])
  async delayed() {
    return await this.backupQueue.getJobs(['delayed']);
  }

  @ResolveField(() => [Job])
  async completed() {
    return await this.backupQueue.getJobs(['completed']);
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
