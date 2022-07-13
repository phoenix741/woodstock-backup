import { Global, Module } from '@nestjs/common';
import { WorkerType, WORKER_TYPE } from '@woodstock/shared';

@Global()
@Module({
  providers: [
    {
      provide: WORKER_TYPE,
      useValue: WorkerType.api,
    },
  ],
  exports: [WORKER_TYPE],
})
export class GlobalModule {}
