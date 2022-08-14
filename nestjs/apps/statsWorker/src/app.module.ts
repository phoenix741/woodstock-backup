import { Module } from '@nestjs/common';
import { ApplicationConfigModule, SharedModule, WoodstockBullModules } from '@woodstock/shared';
import { GlobalModule } from './global.module.js';
import { StatsConsumer } from './stats/stats.consumer.js';

@Module({
  imports: [GlobalModule, ApplicationConfigModule, ...WoodstockBullModules, SharedModule],
  providers: [StatsConsumer],
})
export class AppModule {}
