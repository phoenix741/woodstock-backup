import { Module } from '@nestjs/common';
import { PoolService } from './pool.service';

@Module({
  providers: [PoolService],
})
export class PoolModule {}
