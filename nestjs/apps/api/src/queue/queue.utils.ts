import { Injectable } from '@nestjs/common';
import {
  Job,
  JobBackupData,
  JobGroupTasks,
  JobSubTask,
  QueueGroupTasks,
  QueueSubTask,
  QueueTasksService,
  RefcntJobData,
  TaskLocalContext,
} from '@woodstock/server';
import Bull from 'bullmq';
import { plainToInstance } from 'class-transformer';

@Injectable()
export class QueueUtils {
  constructor(private queueTasksService: QueueTasksService) {}

  #getDescription(localContext: TaskLocalContext) {
    const sharePath = localContext.sharePath?.toString('utf-8');
    const host = localContext.host;
    const number = localContext.number;

    return [sharePath, host, number].filter((v) => v !== undefined).join(' - ');
  }

  #getTask(subtask: QueueSubTask): JobSubTask {
    return plainToInstance(JobSubTask, {
      taskName: subtask.taskName,
      state: subtask.state,
      progression: subtask.progression,
      description: this.#getDescription(subtask.localContext),
    });
  }

  #isGroupTask(subtask: QueueSubTask | QueueGroupTasks): subtask is QueueGroupTasks {
    return (subtask as QueueGroupTasks).groupName !== undefined;
  }

  #getJobGroupTask(group: QueueGroupTasks): JobGroupTasks {
    return plainToInstance(JobGroupTasks, {
      groupName: group.groupName,
      state: group.state,
      progression: group.progression,
      subtasks: this.#getJobGroupTasks(group.subtasks),
      description: this.#getDescription(group.localContext),
    });
  }

  #getJobGroupTasks(subtasks: (QueueSubTask | QueueGroupTasks)[]): (JobSubTask | JobGroupTasks)[] {
    return (subtasks || []).map((subtask) => {
      if (this.#isGroupTask(subtask)) {
        return this.#getJobGroupTask(subtask);
      } else {
        return this.#getTask(subtask);
      }
    });
  }

  async getJob(job: Bull.Job<JobBackupData | RefcntJobData>): Promise<Job> {
    const progress = this.queueTasksService.deserializeBackupTask(job.progress as object);
    const subtasks = this.#getJobGroupTasks(progress.subtasks);

    return plainToInstance(Job, {
      id: job.id,
      queueName: job.queueName,
      name: job.name,
      state: await job.getState(),

      data: {
        host: job.data.host,
        number: job.data.number,
        ip: (job.data as JobBackupData).ip,
        startDate: job.processedOn ?? job.timestamp,

        state: progress.state,
        progression: progress.progression,
        subtasks,
      },

      attemptsMade: job.attemptsMade,
      failedReason: job.failedReason,
    });
  }
}
