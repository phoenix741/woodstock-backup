import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { ApplicationConfigModule, SharedModule, DiskStatisticsService, WoodstockBullModules } from '@woodstock/shared';
import { ConsoleModule } from 'nestjs-console';
import { BackupsCommand } from './backups/backups.command';
import { StatsCommand } from './stats/stats.command';

@Module({
  imports: [ApplicationConfigModule, ...WoodstockBullModules, ConfigModule.forRoot(), ConsoleModule, SharedModule],
  providers: [BackupsCommand, StatsCommand, DiskStatisticsService],
})
export class AppCommandModule {}
