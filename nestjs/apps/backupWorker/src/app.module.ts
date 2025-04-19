import { CacheModule } from '@nestjs/cache-manager';
import { Module, OnApplicationBootstrap } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { IORedisOptions } from '@nestjs/microservices/external/redis.interface.js';
import {
  ApplicationLogger,
  BackupsService,
  CacheConfigService,
  ConfigProviderModule,
  SharedModule,
  initializeLog,
} from '@woodstock/shared';
import { BackupClientProgress } from './backups/backup-client-progress.service.js';
import { BackupsClientService } from './backups/backups-client.service.js';
import { BackupTasksService } from './tasks/backup-tasks.service.js';
import { HostConsumer } from './tasks/host.consumer.js';
import { RemoveService } from './tasks/remove.service.js';
import { RestoreService } from './tasks/restore.service.js';
import { HostConsumerUtilService } from './utils/host-consumer-util.service.js';

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
    RestoreService,
    {
      provide: ApplicationLogger,
      useFactory: (backupsService) => new ApplicationLogger('backup', backupsService),
      inject: [BackupsService],
    },
  ],
})
export class AppModule implements OnApplicationBootstrap {
  async onApplicationBootstrap() {
    await initializeLog();
  }
}
