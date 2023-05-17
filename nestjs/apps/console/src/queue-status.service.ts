import { QueueEventsHost, QueueEventsListener } from '@nestjs/bullmq';
import { Injectable } from '@nestjs/common';
import { JobBackupData, QueueName, RefcntJobData } from '@woodstock/server';
import { QueueTasks, QueueTasksService } from '@woodstock/server/tasks';
import { Job } from 'bullmq';
import { Constructor } from 'protobufjs';
import { Observable } from 'rxjs';

export interface QueueStatusInterface<T = unknown> {
  waitingJob(job: Job<T>): Observable<QueueTasks>;
}

export function QueueStatus<T = unknown>(queueName: QueueName): Constructor<QueueStatusInterface<T>> {
  @Injectable()
  @QueueEventsListener(queueName)
  class QueueStatus extends QueueEventsHost implements QueueStatusInterface<T> {
    constructor(private queueTasksService: QueueTasksService) {
      super();
    }

    waitingJob(job: Job<T>): Observable<QueueTasks> {
      const jobId = job.id;
      return new Observable((observer) => {
        const onProgress = async ({ data }: { jobId: number; data: object }) => {
          const tasks = this.queueTasksService.deserializeBackupTask(data);
          observer.next(tasks);
        };
        const progressEvent = `progress:${jobId}`;
        this.queueEvents.on(progressEvent as any, onProgress);
        const removeListeners = () => {
          this.queueEvents.removeListener(progressEvent, onProgress);
        };

        job
          .waitUntilFinished(this.queueEvents)
          .then(() => {
            observer.complete();
            removeListeners();
          })
          .catch((err) => {
            observer.error(err);
            removeListeners();
          });
      });
    }
  }

  return QueueStatus;
}

export const RefcntQueueStatus = QueueStatus<RefcntJobData>(QueueName.REFCNT_QUEUE);
export const BackupQueueStatus = QueueStatus<JobBackupData>(QueueName.BACKUP_QUEUE);
