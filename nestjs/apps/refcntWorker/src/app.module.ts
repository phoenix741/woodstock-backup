import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { CommandsModule, CoreModule, QueueModule } from '@woodstock/shared';
import { GlobalModule } from './global.module.js';
import { RefcntConsumer } from './refcnt.consumer.js';

@Module({
  imports: [ConfigModule.forRoot(), GlobalModule, CoreModule, CommandsModule, QueueModule],
  providers: [RefcntConsumer],
})
export class AppModule {}
