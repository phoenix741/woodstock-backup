import { Module, OnApplicationBootstrap } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import {
  ApplicationConfigService,
  CacheConfigService,
  ConfigProviderModule,
  SharedModule,
  initializeLog,
} from '@woodstock/shared';
import { BackupTasksService } from './tasks/backup-tasks.service.js';
import { HostConsumer } from './tasks/host.consumer.js';
import { RemoveService } from './tasks/remove.service.js';
import { HostConsumerUtilService } from './utils/host-consumer-util.service.js';
import { BackupsClientService } from './backups/backups-client.service.js';
import { BackupClientProgress } from './backups/backup-client-progress.service.js';
import { CacheModule } from '@nestjs/cache-manager';
import { IORedisOptions } from '@nestjs/microservices/external/redis.interface.js';

@Module({
  imports: [
    ConfigModule.forRoot({ isGlobal: true }),
    CacheModule.registerAsync<IORedisOptions>({
      isGlobal: true,
      useClass: CacheConfigService,
      imports: [ConfigProviderModule],
    }),
    ConfigProviderModule,
    SharedModule,
  ],
  providers: [
    BackupClientProgress,
    BackupsClientService,
    CacheConfigService,
    HostConsumer,
    HostConsumerUtilService,
    BackupTasksService,
    RemoveService,
  ],
})
export class AppModule implements OnApplicationBootstrap {
  constructor(private readonly config: ApplicationConfigService) {}

  async onApplicationBootstrap() {
    await initializeLog(this.config.context);
  }
}
