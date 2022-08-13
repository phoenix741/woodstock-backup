import { Module } from '@nestjs/common';
import { ApplicationConfigModule, SharedModule, WoodstockBullModules } from '@woodstock/shared';
import { GlobalModule } from './global.module.js';
import { RefcntConsumer } from './refcnt.consumer.js';

@Module({
  imports: [GlobalModule, ApplicationConfigModule, ...WoodstockBullModules, SharedModule],
  providers: [RefcntConsumer],
})
export class AppModule {}
