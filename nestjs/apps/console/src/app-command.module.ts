import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { ApplicationConfigModule, SharedModule, WoodstockBullModules } from '@woodstock/shared';
import { ConsoleModule } from 'nestjs-console';
import { BackupsCommand } from './backups/backups.command';
import { GlobalModule } from './global.module';
import { PoolCommand } from './pool/pool.command';
import { ProtobufCommand } from './protobuf/protobuf.command';
import { StatsCommand } from './stats/stats.command';

@Module({
  imports: [
    GlobalModule,
    ApplicationConfigModule,
    ...WoodstockBullModules,
    ConfigModule.forRoot(),
    ConsoleModule,
    SharedModule,
  ],
  providers: [BackupsCommand, StatsCommand, PoolCommand, ProtobufCommand],
})
export class AppCommandModule {}
