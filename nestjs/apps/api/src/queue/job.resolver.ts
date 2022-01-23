import { InjectQueue } from '@nestjs/bull';
import { Float, Parent, ResolveField, Resolver } from '@nestjs/graphql';
import { BackupTask, Job } from '@woodstock/backoffice-shared';
import { JobStatus, Queue } from 'bull';

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
