import { Module } from '@nestjs/common';
import { ApplicationConfigModule, WoodstockBullModules } from '@woodstock/shared';
import { SharedModule } from '@woodstock/shared';
import { StatsConsumer } from './stats/stats.consumer';

@Module({
  imports: [ApplicationConfigModule, ...WoodstockBullModules, SharedModule],
  providers: [StatsConsumer],
})
export class AppModule {}
