import { InjectQueue } from '@nestjs/bull';
import { ResolveField, Resolver, Parent, Float } from '@nestjs/graphql';
import { Queue, JobStatus } from 'bull';

import { Job, BackupTask } from '../tasks/tasks.dto';

@Resolver(() => Job)
export class JobResolver {
  constructor(@InjectQueue('queue') private backupQueue: Queue<BackupTask>) {}

  @ResolveField(() => String, { nullable: true })
  async state(@Parent() job: Job): Promise<JobStatus | 'stuck' | undefined> {
    return await (await this.backupQueue.getJob(job.id))?.getState();
  }

  @ResolveField(() => Float, { nullable: true })
  async progress(@Parent() job: Job): Promise<number> {
    return (await this.backupQueue.getJob(job.id))?.progress();
  }
}
