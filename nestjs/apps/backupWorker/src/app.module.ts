import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { CoreModule } from '@woodstock/core';
import { ServerModule } from '@woodstock/server';
import { SharedModule } from '@woodstock/shared';
import { BackupClientGrpc } from './backups/backup-client-grpc.class.js';
import { BackupClientLocal } from './backups/backup-client-local.class.js';
import { BackupClientProgress } from './backups/backup-client-progress.service.js';
import { BackupClient } from './backups/backup-client.service.js';
import { GlobalModule } from './global.module.js';
import { BackupTasksService } from './tasks/backup-tasks.service.js';
import { HostConsumer } from './tasks/host.consumer.js';
import { RemoveService } from './tasks/remove.service.js';
import { HostConsumerUtilService } from './utils/host-consumer-util.service.js';

@Module({
  imports: [ConfigModule.forRoot(), GlobalModule, CoreModule, SharedModule, ServerModule],
  providers: [
    BackupClientProgress,
    BackupClient,
    BackupClientGrpc,
    BackupClientLocal,
    HostConsumer,
    HostConsumerUtilService,
    BackupTasksService,
    RemoveService,
  ],
})
export class AppModule {}
