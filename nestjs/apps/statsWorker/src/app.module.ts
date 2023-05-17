import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { CoreModule } from '@woodstock/core';
import { ServerModule } from '@woodstock/server';
import { SharedModule } from '@woodstock/shared';
import { GlobalModule } from './global.module.js';
import { StatsConsumer } from './stats/stats.consumer.js';

@Module({
  imports: [ConfigModule.forRoot(), GlobalModule, CoreModule, SharedModule, ServerModule],
  providers: [StatsConsumer],
})
export class AppModule {}
