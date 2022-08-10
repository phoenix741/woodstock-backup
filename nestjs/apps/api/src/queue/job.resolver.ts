import { InjectQueue } from '@nestjs/bullmq';
import { Float, Parent, ResolveField, Resolver } from '@nestjs/graphql';
import { BackupTask, Job } from '@woodstock/shared';
import { JobState, Queue } from 'bullmq';

@Resolver(() => Job)
export class JobResolver {
  constructor(@InjectQueue('queue') private backupQueue: Queue<BackupTask>) {}

  @ResolveField(() => String, { nullable: true })
  async state(@Parent() job: Job): Promise<JobState | 'unknown' | undefined> {
    return await (await this.backupQueue.getJob(job.id))?.getState();
  }

  @ResolveField(() => Float, { nullable: true })
  async progress(@Parent() job: Job): Promise<number | undefined> {
    const progress = (await this.backupQueue.getJob(job.id))?.progress;
    if (typeof progress === 'number') {
      return progress;
    }
    return undefined;
  }
}
