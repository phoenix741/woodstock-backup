import { InjectQueue } from '@nestjs/bullmq';
import { NotFoundException } from '@nestjs/common';
import { Args, Mutation, Resolver } from '@nestjs/graphql';
import { BackupQueueData, QueueName } from '@woodstock/shared';
import { Queue } from 'bullmq';
import { JobResponse } from '../backups/backups.dto';

@Resolver()
export class PoolResolver {
  constructor(@InjectQueue(QueueName.BACKUP_QUEUE) private backupQueue: Queue<BackupQueueData>) {}

  @Mutation(() => JobResponse)
  async cleanupPool(): Promise<JobResponse> {
    const { id } = await this.backupQueue.add('cleanup_refcnt', {});
    if (!id) {
      throw new NotFoundException(`Can't cleanup the pool`);
    }

    return {
      id,
    };
  }

  @Mutation(() => JobResponse)
  async checkAndFixPool(
    @Args('fix', { type: () => Boolean }) fix?: boolean,
    @Args('verifyChunks', { type: () => Boolean }) verifyChunks?: boolean,
  ): Promise<JobResponse> {
    const { id } = await this.backupQueue.add('fsck', { dryRun: !fix, verifyChunks: !!verifyChunks });
    if (!id) {
      throw new NotFoundException(`Can't check and fix the pool`);
    }

    return {
      id,
    };
  }
}
