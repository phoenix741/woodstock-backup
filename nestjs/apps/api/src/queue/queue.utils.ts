import { Injectable } from '@nestjs/common';
import {
  BackupProgressData,
  BackupQueueData,
  BackupTaskState,
  CleanerTaskState,
  FsckTaskState,
  Job,
  JobBackupData,
  JobCleanupData,
  JobFsckData,
  JobRemoveData,
  JobRestoreData,
  QueueTasksService,
  RemoveTaskState,
  RestoreTaskState,
} from '@woodstock/shared';
import Bull from 'bullmq';
import { plainToInstance } from 'class-transformer';

@Injectable()
export class QueueUtils {
  constructor(private queueTasksService: QueueTasksService) {}

  #getJobData(job: Bull.Job<BackupQueueData>): BackupQueueData {
    switch (job.name) {
      case 'backup':
        return this.queueTasksService.deserializeBackupTask(job.data, JobBackupData);
      case 'restore':
        return this.queueTasksService.deserializeBackupTask(job.data, JobRestoreData);
      case 'remove_backup':
        return this.queueTasksService.deserializeBackupTask(job.data, JobRemoveData);
      case 'cleanup_refcnt':
        return this.queueTasksService.deserializeBackupTask(job.data, JobCleanupData);
      case 'fsck':
        return this.queueTasksService.deserializeBackupTask(job.data, JobFsckData);
      default:
        throw new Error(`Unknown job name: ${job.name}`);
    }
  }

  #getJobProgress(job: Bull.Job<BackupQueueData>): BackupProgressData | undefined {
    if (!job.progress) {
      return undefined;
    }
    switch (job.name) {
      case 'backup':
        return this.queueTasksService.deserializeBackupTask(job.progress as object, BackupTaskState);
      case 'restore':
        return this.queueTasksService.deserializeBackupTask(job.progress as object, RestoreTaskState);
      case 'remove_backup':
        return this.queueTasksService.deserializeBackupTask(job.progress as object, RemoveTaskState);
      case 'cleanup_refcnt':
        return this.queueTasksService.deserializeBackupTask(job.progress as object, CleanerTaskState);
      case 'fsck':
        return this.queueTasksService.deserializeBackupTask(job.progress as object, FsckTaskState);
      default:
        throw new Error(`Unknown job name: ${job.name}`);
    }
  }

  async getJob(job: Bull.Job<BackupQueueData>): Promise<Job> {
    const data = this.#getJobData(job);
    const progression = this.#getJobProgress(job);

    return plainToInstance(Job, {
      id: job.id,
      queueName: job.queueName,
      name: job.name,
      state: await job.getState(),

      data,
      progression,

      attemptsMade: job.attemptsMade,
      failedReason: job.failedReason,
    });
  }
}
