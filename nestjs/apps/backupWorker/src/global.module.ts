import { Global, Module } from '@nestjs/common';
import { WorkerType, WORKER_TYPE } from '@woodstock/core';

@Global()
@Module({
  providers: [
    {
      provide: WORKER_TYPE,
      useValue: WorkerType.backupWorker,
    },
  ],
  exports: [WORKER_TYPE],
})
export class GlobalModule {}
