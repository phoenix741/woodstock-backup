import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { CoreModule } from '@woodstock/shared';
import { ConsoleModule } from 'nestjs-console';
import { GlobalModule } from './global.module.js';
import { QueueCommandModule } from './queue-command/queue-command.module.js';
import { StdCommandModule } from './std-command/std-command.module.js';

@Module({
  imports: [ConfigModule.forRoot(), GlobalModule, CoreModule, ConsoleModule, StdCommandModule, QueueCommandModule],
})
export class AppCommandModule {}
