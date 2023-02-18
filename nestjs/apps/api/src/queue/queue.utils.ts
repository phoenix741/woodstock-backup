import { Injectable } from '@nestjs/common';
import { Job, JobGroupTasks, JobSubTask } from '@woodstock/shared';
import { BackupNameTask, BackupShareContext, JobBackupData } from '@woodstock/shared/backuping/backuping.model.js';
import { QueueGroupTasks, QueueSubTask, QueueTasks } from '@woodstock/shared/tasks';
import Bull from 'bullmq';
import { plainToInstance } from 'class-transformer';

@Injectable()
export class QueueUtils {
  #getTask(subtask: QueueSubTask): JobSubTask {
    let description: string;
    switch (subtask.taskName) {
      case BackupNameTask.FILELIST_TASK:
      case BackupNameTask.CHUNKS_TASK:
      case BackupNameTask.COMPACT_TASK:
        description = (subtask.localContext as BackupShareContext).sharePath.toString();
        break;
      default:
        description = '';
    }

    return plainToInstance(JobSubTask, {
      taskName: subtask.taskName,
      state: subtask.state,
      progression: subtask.progression,
      description,
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
      description: '',
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

  async getJob(job: Bull.Job<JobBackupData>): Promise<Job> {
    const progress = job.progress as QueueTasks;
    return plainToInstance(Job, {
      id: job.id,
      name: job.name,
      state: await job.getState(),

      data: {
        host: job.data.host,
        number: job.data.number,
        ip: job.data.ip,
        startDate: job.data.startDate,

        state: progress.state,
        progression: progress.progression,
        subtasks: this.#getJobGroupTasks(progress.subtasks),
      },

      attemptsMade: job.attemptsMade,
    });
  }
}
