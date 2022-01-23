import { Module } from '@nestjs/common';
import { ApplicationConfigModule, BackofficeSharedModule, WoodstockBullModules } from '@woodstock/backoffice-shared';
import { SharedModule } from '@woodstock/shared';
import { StatsConsumer } from './stats/stats.consumer';

@Module({
  imports: [ApplicationConfigModule, BackofficeSharedModule, ...WoodstockBullModules, SharedModule],
  providers: [StatsConsumer],
})
export class AppModule {}
