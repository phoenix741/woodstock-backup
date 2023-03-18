import { InjectQueue } from '@nestjs/bullmq';
import { NotFoundException } from '@nestjs/common';
import { Args, Mutation, Resolver } from '@nestjs/graphql';
import { QueueName, RefcntJobData } from '@woodstock/shared';
import { Queue } from 'bullmq';
import { JobResponse } from '../backups/backups.dto';

@Resolver()
export class PoolResolver {
  constructor(@InjectQueue(QueueName.REFCNT_QUEUE) private refcntQueue: Queue<RefcntJobData>) {}

  @Mutation(() => JobResponse)
  async cleanupPool(): Promise<JobResponse> {
    const { id } = await this.refcntQueue.add('unused', {});
    if (!id) {
      throw new NotFoundException(`Can't cleanup the pool`);
    }

    return {
      id,
    };
  }

  @Mutation(() => JobResponse)
  async checkAndFixPool(@Args('fix', { type: () => Boolean }) fix?: boolean): Promise<JobResponse> {
    const { id } = await this.refcntQueue.add('fsck', { fix, refcnt: true, unused: true });
    if (!id) {
      throw new NotFoundException(`Can't check and fix the pool`);
    }

    return {
      id,
    };
  }

  @Mutation(() => JobResponse)
  async verifyChecksum(): Promise<JobResponse> {
    const { id } = await this.refcntQueue.add('verify_checksum', {});
    if (!id) {
      throw new NotFoundException(`Can't verify checksum the pool`);
    }

    return {
      id,
    };
  }
}
