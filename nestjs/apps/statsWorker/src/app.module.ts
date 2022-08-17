import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { CommandsModule, CoreModule, QueueModule } from '@woodstock/shared';
import { GlobalModule } from './global.module.js';
import { StatsConsumer } from './stats/stats.consumer.js';

@Module({
  imports: [ConfigModule.forRoot(), GlobalModule, CoreModule, CommandsModule, QueueModule],
  providers: [StatsConsumer],
})
export class AppModule {}
