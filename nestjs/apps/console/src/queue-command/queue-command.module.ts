import { Module } from '@nestjs/common';
import { ApplicationConfigModule, PoolModule, QueueModule } from '@woodstock/shared';
import { BackupsCommand } from './backups.command';
import { StatsCommand } from './stats.command';

@Module({
  imports: [QueueModule, ApplicationConfigModule, PoolModule],
  providers: [BackupsCommand, StatsCommand],
})
export class QueueCommandModule {}
