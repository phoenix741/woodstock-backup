import { Module } from '@nestjs/common';
import { CommandsModule } from '../commands';
import { ApplicationConfigModule } from '../config';
import { InputOutputModule } from '../input-output';
import { DiskStatisticsService } from './disk-statistics.service';
import { PoolStatisticsService } from './pool-statistics.service';
import { StatsInstantService } from './stats-instant.service';

@Module({
  imports: [InputOutputModule, ApplicationConfigModule, CommandsModule],
  providers: [DiskStatisticsService, PoolStatisticsService, StatsInstantService],
  exports: [DiskStatisticsService, PoolStatisticsService, StatsInstantService],
})
export class StatisticsModule {}
