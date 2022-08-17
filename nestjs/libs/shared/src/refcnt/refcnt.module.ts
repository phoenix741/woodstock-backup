import { Module } from '@nestjs/common';
import { InputOutputModule } from '../input-output';
import { StatisticsModule } from '../statistics/statistics.module';
import { RefCntService } from './refcnt.service';

@Module({
  imports: [InputOutputModule, StatisticsModule],
  providers: [RefCntService],
  exports: [RefCntService],
})
export class RefcntModule {}
