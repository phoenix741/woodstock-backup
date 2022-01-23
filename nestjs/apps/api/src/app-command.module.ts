import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import {
  ApplicationConfigModule,
  BackofficeSharedModule,
  StatsService,
  WoodstockBullModules,
} from '@woodstock/backoffice-shared';
import { SharedModule } from '@woodstock/shared';
import { ConsoleModule } from 'nestjs-console';
import { BackupsCommand } from './backups/backups.command';
import { StatsCommand } from './stats/stats.command';

@Module({
  imports: [
    ApplicationConfigModule,
    BackofficeSharedModule,
    ...WoodstockBullModules,
    ConfigModule.forRoot(),
    ConsoleModule,
    SharedModule,
  ],
  providers: [BackupsCommand, StatsCommand, StatsService],
})
export class AppCommandModule {}
