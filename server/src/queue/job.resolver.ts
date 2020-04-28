import { InjectQueue } from '@nestjs/bull';
import { ResolveField, Resolver, Parent } from '@nestjs/graphql';
import { Queue } from 'bull';

import { Job, BackupTask } from '../tasks/tasks.dto';

@Resolver(() => Job)
export class JobResolver {
  constructor(@InjectQueue('queue') private backupQueue: Queue<BackupTask>) {}

  @ResolveField(() => String, { nullable: true })
  async state(@Parent() job: Job) {
    return await (await this.backupQueue.getJob(job.id))?.getState();
  }
}
