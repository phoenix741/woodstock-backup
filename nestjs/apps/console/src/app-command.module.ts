import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { ApplicationConfigModule, SharedModule, WoodstockBullModules } from '@woodstock/shared';
import { ConsoleModule } from 'nestjs-console';
import { BackupsCommand } from './backups/backups.command.js';
import { BrowserCommand } from './backups/browser.command.js';
import { GlobalModule } from './global.module.js';
import { PoolCommand } from './pool/pool.command.js';
import { ProtobufCommand } from './protobuf/protobuf.command.js';
import { StatsCommand } from './stats/stats.command.js';

@Module({
  imports: [
    GlobalModule,
    ApplicationConfigModule,
    ...WoodstockBullModules,
    ConfigModule.forRoot(),
    ConsoleModule,
    SharedModule,
  ],
  providers: [BackupsCommand, StatsCommand, PoolCommand, ProtobufCommand, BrowserCommand],
})
export class AppCommandModule {}
