import { Module } from '@nestjs/common';
import { ApplicationConfigModule, SharedModule, WoodstockBullModules } from '@woodstock/shared';
import { BackupClientGrpc } from './backups/backup-client-grpc.class';
import { BackupClientProgress } from './backups/backup-client-progress.service';
import { BackupClient } from './backups/backup-client.service';
import { HostConsumer } from './tasks/host.consumer';
import { TasksService } from './tasks/tasks.service';
import { HostConsumerUtilService } from './utils/host-consumer-util.service';

@Module({
  imports: [ApplicationConfigModule, ...WoodstockBullModules, SharedModule],
  providers: [
    BackupClientProgress,
    BackupClient,
    BackupClientGrpc,
    HostConsumer,
    HostConsumerUtilService,
    TasksService,
  ],
})
export class AppModule {}
