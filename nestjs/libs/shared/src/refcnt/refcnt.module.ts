import { Module } from '@nestjs/common';
import { InputOutputModule } from '../input-output';
import { SharedModule } from '../shared/shared.module';
import { StatisticsModule } from '../statistics/statistics.module';
import { RefCntService } from './refcnt.service';

@Module({
  imports: [InputOutputModule, StatisticsModule, SharedModule],
  providers: [RefCntService],
  exports: [RefCntService],
})
export class RefcntModule {}
