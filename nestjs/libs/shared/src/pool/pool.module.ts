import { Module } from '@nestjs/common';
import { ApplicationConfigModule } from '../config';
import { RefcntModule } from '../refcnt';
import { ScannerModule } from '../scanner';
import { PoolService } from './pool.service';

@Module({
  imports: [ApplicationConfigModule, ScannerModule, RefcntModule],
  providers: [PoolService],
  exports: [PoolService],
})
export class PoolModule {}
