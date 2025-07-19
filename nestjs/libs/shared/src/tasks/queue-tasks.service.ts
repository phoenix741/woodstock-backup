import { Injectable, Type } from '@nestjs/common';
import { SandboxedJob } from 'bullmq';
import { instanceToPlain, plainToInstance } from 'class-transformer';
import { concatMap, lastValueFrom, Observable, throttleTime } from 'rxjs';

@Injectable()
export class QueueTasksService {
  async processJobData<JobData, Context>(
    job: SandboxedJob<JobData>,
    observable$: Observable<Context>,
  ): Promise<Context> {
    const lastValue = await lastValueFrom(
      observable$.pipe(
        throttleTime(1000, undefined, { leading: true, trailing: true }),
        concatMap(async (task) => {
          // Convertir l'état si nécessaire avant de le sérialiser
          const stateToSave = this.serializeBackupTask(task);
          job.updateProgress(stateToSave);
          return task;
        }),
      ),
    );

    // Mettre à jour la progression avec la dernière valeur
    const finalState = this.serializeBackupTask(lastValue);
    job.updateProgress(finalState);

    return lastValue;
  }

  /**
   * Sérialise une tâche pour la mettre dans la progression de la file d'attente
   */
  serializeBackupTask<JobData>(tasks: JobData): object {
    return instanceToPlain(tasks);
  }

  /**
   * Désérialise une tâche à partir de la progression de la file d'attente
   */
  deserializeBackupTask<JobData>(data: object, cstor?: Type<JobData>): JobData {
    if (cstor) {
      return plainToInstance(cstor, data);
    }
    return data as JobData;
  }
}
