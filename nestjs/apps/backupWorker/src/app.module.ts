import { CacheModule } from '@nestjs/cache-manager';
import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { IORedisOptions } from '@nestjs/microservices/external/redis.interface.js';
import { CacheConfigService, ConfigProviderModule, SharedModule } from '@woodstock/shared';
import { BackupLogger } from './backup.logger.js';
import { BackupMachineService } from './backups/backup-machine.service.js';
import { RemoveMachineService } from './backups/remove-machine.service.js';
import { RestoreMachineService } from './backups/restore-machine.service.js';
import { CleanupMachineService } from './pool/cleanup-machine.service.js';
import { FsckMachineService } from './pool/fsck-machine.service.js';
import { HostConsumer } from './tasks/host.consumer.js';
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
    CacheConfigService,
    HostConsumer,
    HostConsumerUtilService,
    BackupMachineService,
    RestoreMachineService,
    RemoveMachineService,
    CleanupMachineService,
    FsckMachineService,
    BackupLogger,
  ],
})
export class AppModule {}
