import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { ConfigProviderModule, SharedModule } from '@woodstock/shared';
import { StatsConsumer } from './stats/stats.consumer.js';

@Module({
  imports: [ConfigModule.forRoot({ isGlobal: true }), ConfigProviderModule, SharedModule],
  providers: [StatsConsumer],
})
export class AppModule {}
