import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { AuthentificationModule, BackupingModule, CommandsModule, CoreModule, QueueModule } from '@woodstock/shared';
import { BackupClientModule } from '@woodstock/shared/client/backup-client.module.js';
import { QueueTasksModule } from '@woodstock/shared/tasks/queue-tasks.module.js';
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
  imports: [
    ConfigModule.forRoot(),
    GlobalModule,
    CoreModule,
    CommandsModule,
    QueueModule,
    AuthentificationModule,
    BackupingModule,
    QueueTasksModule,
    BackupClientModule,
  ],
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
