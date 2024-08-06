import { Module, OnApplicationBootstrap } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { ApplicationConfigService, ConfigProviderModule, SharedModule, initializeLog } from '@woodstock/shared';
import { BackupTasksService } from './tasks/backup-tasks.service.js';
import { HostConsumer } from './tasks/host.consumer.js';
import { RemoveService } from './tasks/remove.service.js';
import { HostConsumerUtilService } from './utils/host-consumer-util.service.js';
import { BackupsClientService } from './backups/backups-client.service.js';
import { BackupClientProgress } from './backups/backup-client-progress.service.js';

@Module({
  imports: [ConfigModule.forRoot({ isGlobal: true }), ConfigProviderModule, SharedModule],
  providers: [
    BackupClientProgress,
    BackupsClientService,
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