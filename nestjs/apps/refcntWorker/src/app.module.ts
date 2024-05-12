import { Module, OnApplicationBootstrap } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { ApplicationConfigService, ConfigProviderModule, initializeLog, SharedModule } from '@woodstock/shared';
import { RefcntConsumer } from './refcnt.consumer.js';

@Module({
  imports: [ConfigModule.forRoot({ isGlobal: true }), ConfigProviderModule, SharedModule],
  providers: [RefcntConsumer],
})
export class AppModule implements OnApplicationBootstrap {
  constructor(private readonly config: ApplicationConfigService) {}

  async onApplicationBootstrap() {
    await initializeLog(this.config.context);
  }
}
