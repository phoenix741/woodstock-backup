import { Module } from '@nestjs/common';
import { ApplicationConfigModule, SharedModule, WoodstockBullModules } from '@woodstock/shared';
import { BackupClientGrpc } from './backups/backup-client-grpc.class.js';
import { BackupClientProgress } from './backups/backup-client-progress.service.js';
import { BackupClient } from './backups/backup-client.service.js';
import { GlobalModule } from './global.module.js';
import { HostConsumer } from './tasks/host.consumer.js';
import { RemoveService } from './tasks/remove.service.js';
import { TasksService } from './tasks/tasks.service.js';
import { HostConsumerUtilService } from './utils/host-consumer-util.service.js';

@Module({
  imports: [GlobalModule, ApplicationConfigModule, ...WoodstockBullModules, SharedModule],
  providers: [
    BackupClientProgress,
    BackupClient,
    BackupClientGrpc,
    HostConsumer,
    HostConsumerUtilService,
    TasksService,
    RemoveService,
  ],
})
export class AppModule {}
